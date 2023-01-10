import CryptoKit
import Darwin
import Foundation

class Plugin {
  var stream: Stream

  init(stream: Stream) {
    self.stream = stream
  }

  func generateKey(outputFile: String? = nil, accessControl: KeyAccessControl) throws {
    if !SecureEnclave.isAvailable {
      throw Error.seUnsupported
    }
    let createdAt = Date().ISO8601Format()
    var accessControlFlags: SecAccessControlCreateFlags = [.privateKeyUsage]
    if accessControl == .biometry || accessControl == .biometryAndPasscode {
      accessControlFlags.insert(.biometryAny)
    }
    if accessControl == .passcode || accessControl == .biometryAndPasscode {
      accessControlFlags.insert(.devicePasscode)
    }
    if accessControl == .biometryOrPasscode {
      accessControlFlags.insert(.userPresence)
    }
    let privateKey: SecureEnclave.P256.KeyAgreement.PrivateKey = try SecureEnclave.P256
      .KeyAgreement.PrivateKey(
        accessControl: SecAccessControlCreateWithFlags(
          kCFAllocatorDefault, kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
          accessControlFlags,
          nil)!)
    let publicKey = privateKey.publicKey.compressedRepresentation
    let recipient = Bech32().encode(hrp: "age1applese", data: publicKey)
    let identity = Bech32().encode(
      hrp: "AGE-PLUGIN-APPLESE-",
      data: privateKey.dataRepresentation)

    let accessControlStr: String
    switch accessControl {
    case .none: accessControlStr = "none"
    case .biometry: accessControlStr = "biometry"
    case .passcode: accessControlStr = "passcode"
    case .biometryOrPasscode: accessControlStr = "biometry or passcode"
    case .biometryAndPasscode: accessControlStr = "biometry and passcode"
    }

    let contents = """
      # created: \(createdAt)
      # access control: \(accessControlStr)
      # public key: \(recipient)
      \(identity)
      """

    if let outputFile = outputFile {
      FileManager.default.createFile(
        atPath: FileManager.default.currentDirectoryPath + "/" + outputFile,
        contents: contents.data(using: .utf8),
        attributes: [.posixPermissions: 0o600]
      )
      print("Public key: \(recipient)")
    } else {
      print(contents)
    }
  }

  func runRecipientV1() {
    var recipients: [String] = []
    var identities: [String] = []
    var fileKeys: [Data] = []

    // Phase 1
    loop: while true {
      let stanza = try! Stanza.readFrom(stream: stream)
      switch stanza.type {
      case "add-recipient":
        recipients.append(stanza.args[0])
      case "add-identity":
        identities.append(stanza.args[0])
      case "wrap-file-key":
        fileKeys.append(stanza.body)
      case "done":
        break loop
      default:
        continue
      }
    }

    // Phase 2
    var responses: [Stanza] = []
    var recipientKeys: [P256.KeyAgreement.PublicKey] = []
    recipients.enumerated().forEach { (index, recipient) in
      do {
        let id = try Bech32().decode(recipient)
        if id.hrp != "age1applese" {
          throw Error.unknownHRP(id.hrp)
        }
        recipientKeys.append(try P256.KeyAgreement.PublicKey(compressedRepresentation: id.data))
      } catch {
        responses.append(
          Stanza(error: "recipient", args: [String(index)], message: "\(error)"))
      }
    }
    identities.enumerated().forEach { (index, identity) in
      do {
        let id = try Bech32().decode(identity)
        if id.hrp != "AGE-PLUGIN-APPLESE-" {
          throw Error.unknownHRP(id.hrp)
        }
        recipientKeys.append(
          (try SecureEnclave.P256.KeyAgreement.PrivateKey(dataRepresentation: id.data))
            .publicKey)
      } catch {
        responses.append(
          Stanza(error: "recipient", args: [String(index)], message: "\(error)"))
      }
    }
    fileKeys.enumerated().forEach { (index, fileKey) in
      for recipientKey in recipientKeys {
        do {
          let ephemeralSecretKey = P256.KeyAgreement.PrivateKey()
          let ephemeralPublicKeyBytes = ephemeralSecretKey.publicKey.compressedRepresentation
          let sharedSecret = try ephemeralSecretKey.sharedSecretFromKeyAgreement(with: recipientKey)
          let salt = ephemeralPublicKeyBytes + recipientKey.compressedRepresentation
          let wrapKey = sharedSecret.hkdfDerivedSymmetricKey(
            using: SHA256.self, salt: salt,
            sharedInfo: "piv-p256".data(using: .utf8)!,
            outputByteCount: 32
          )
          let body = try ChaChaPoly.seal(fileKey, using: wrapKey).combined
          responses.append(
            Stanza(
              type: "recipient-stanza",
              args: [
                String(index),
                "piv-p256",
                recipientKey.tag.base64RawEncodedString,
                ephemeralPublicKeyBytes.base64RawEncodedString,
              ], body: body
            ))
        } catch {
          responses.append(
            Stanza(error: "internal", args: [], message: "inter"))
        }
      }
    }
    for stanza in responses {
      stanza.writeTo(stream: stream)
      let resp = try! Stanza.readFrom(stream: stream)
      assert(resp.type == "ok")
    }
    Stanza(type: "done").writeTo(stream: stream)
  }

  func runIdentityV1() {
    // Phase 1
    var identities: [String] = []
    var recipientStanzas: [Stanza] = []
    loop: while true {
      let stanza = try! Stanza.readFrom(stream: stream)
      switch stanza.type {
      case "add-identity":
        identities.append(stanza.args[0])
      case "recipient-stanza":
        recipientStanzas.append(stanza)
      case "done":
        break loop
      default:
        continue
      }
    }

    // Phase 2
    var identityKeys: [SecureEnclave.P256.KeyAgreement.PrivateKey] = []
    var responses: [Stanza] = []
    identities.enumerated().forEach { (index, identity) in
      do {
        let id = try Bech32().decode(identity)
        if id.hrp != "AGE-PLUGIN-APPLESE-" {
          throw Error.unknownHRP(id.hrp)
        }
        identityKeys.append(
          (try SecureEnclave.P256.KeyAgreement.PrivateKey(dataRepresentation: id.data)))
      } catch {
        responses.append(
          Stanza(error: "identity", args: [String(index)], message: "\(error)"))
      }
    }
    var handledFiles: Set<String> = []
    recipientStanzas.enumerated().forEach { (index, recipientStanza) in
      let fileIndex = recipientStanza.args[0]
      if handledFiles.contains(fileIndex) {
        return
      }
      if recipientStanza.args.count != 4 {
        return
      }
      let type = recipientStanza.args[1]
      if type != "piv-p256" {
        return
      }
      let tag = recipientStanza.args[2]
      let share = recipientStanza.args[3]
      for identity in identityKeys {
        if identity.publicKey.tag.base64RawEncodedString != tag {
          continue
        }
        do {
          guard let shareKeyData = Data(base64RawEncoded: share) else {
            throw Error.invalidStanza
          }
          let shareKey = try P256.KeyAgreement.PublicKey(compressedRepresentation: shareKeyData)
          let sharedSecret = try identity.sharedSecretFromKeyAgreement(with: shareKey)
          let salt = shareKey.compressedRepresentation + identity.publicKey.compressedRepresentation
          let wrapKey = sharedSecret.hkdfDerivedSymmetricKey(
            using: SHA256.self, salt: salt,
            sharedInfo: "piv-p256".data(using: .utf8)!,
            outputByteCount: 32
          )
          let unwrappedKey = try ChaChaPoly.open(
            ChaChaPoly.SealedBox(combined: recipientStanza.body), using: wrapKey
          )
          responses.append(
            Stanza(
              type: "file-key",
              args: [fileIndex],
              body: unwrappedKey
            ))
          handledFiles.insert(fileIndex)
        } catch {
          // continue
        }
      }
    }

    for stanza in responses {
      stanza.writeTo(stream: stream)
      let resp = try! Stanza.readFrom(stream: stream)
      assert(resp.type == "ok")
    }
    Stanza(type: "done").writeTo(stream: stream)
  }

  enum Error: LocalizedError, Equatable {
    case seUnsupported
    case incompleteStanza
    case invalidStanza
    case unknownHRP(String)

    public var errorDescription: String? {
      switch self {
      case .seUnsupported: return "Secure Enclave not supported on this device"
      case .incompleteStanza: return "Incomplete stanza"
      case .invalidStanza: return "Invalid stanza"
      case .unknownHRP(let hrp): return "Unknown HRP: \(hrp)"
      }
    }
  }
}

//////////////////////////////////////////////////////////////////////////////////////////

struct Stanza: Equatable {
  var type: String
  var args: [String] = []
  var body = Data()

  static func readFrom(stream: Stream) throws -> Stanza {
    guard let header = stream.readLine() else {
      throw Plugin.Error.incompleteStanza
    }
    let headerParts = header.components(separatedBy: " ")
    if headerParts.count < 2 {
      throw Plugin.Error.invalidStanza
    }
    if headerParts[0] != "->" {
      throw Plugin.Error.invalidStanza
    }
    var body = Data()
    while true {
      guard let line = stream.readLine() else {
        throw Plugin.Error.incompleteStanza
      }
      guard let lineData = Data(base64RawEncoded: line) else {
        throw Plugin.Error.invalidStanza
      }
      if lineData.count > 48 {
        throw Plugin.Error.invalidStanza
      }
      body.append(lineData)
      if lineData.count < 48 {
        break
      }
    }
    return Stanza(type: headerParts[1], args: Array(headerParts[2...]), body: body)
  }

  func writeTo(stream: Stream) {
    let parts = ([type] + args).joined(separator: " ")
    stream.writeLine("-> \(parts)\n\(body.base64RawEncodedString)")
  }
}

extension Stanza {
  init(error type: String, args: [String] = [], message: String) {
    self.type = "error"
    self.args = [type] + args
    self.body = message.data(using: .utf8)!
  }
}

enum KeyAccessControl: String {
  case none = "none"
  case biometry = "biometry"
  case passcode = "passcode"
  case biometryOrPasscode = "biometry-or-passcode"
  case biometryAndPasscode = "biometry-and-passcode"
}

extension P256.KeyAgreement.PublicKey {
  var tag: Data {
    return Data(SHA256.hash(data: compressedRepresentation).prefix(4))
  }
}