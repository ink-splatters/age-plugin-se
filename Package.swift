// swift-tools-version: 5.10.1
import PackageDescription

let package = Package(
  name: "AgeSecureEnclavePlugin",
  platforms: [.macOS(.v14)],
  targets: [
    .executableTarget(
      name: "age-plugin-se",
      path: "Sources"),
    .testTarget(name: "Tests", dependencies: ["age-plugin-se"], path: "Tests"),
  ]
)
