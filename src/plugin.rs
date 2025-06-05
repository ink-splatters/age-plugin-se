use crate::crypto::{Crypto, KeyAccessControl, CryptoError, SecureEnclavePrivateKey};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bech32::{encode, decode, ToBase32, FromBase32, Variant};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, AeadCore};
use hmac::{Hmac, Mac};
use p256::{PublicKey, EncodedPoint};
use thiserror::Error;
use std::fmt;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Stream trait for reading and writing lines
pub trait Stream {
    fn read_line(&mut self) -> Option<String>;
    fn write_line(&mut self, line: &str);
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Secure Enclave is not supported on this device")]
    SEUnsupported,
    
    #[error("Incomplete stanza")]
    IncompleteStanza,
    
    #[error("Invalid stanza")]
    InvalidStanza,
    
    #[error("Unknown HRP: {0}")]
    UnknownHRP(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),
    
    #[error("Bech32 error: {0}")]
    Bech32Error(#[from] bech32::Error),
    
    #[error("Base64 decoding error")]
    Base64Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecipientType {
    SE,
    P256Tag,
}

impl fmt::Display for RecipientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipientType::SE => write!(f, "se"),
            RecipientType::P256Tag => write!(f, "p256tag"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecipientStanzaType {
    P256Tag,
    PivP256,
}

impl fmt::Display for RecipientStanzaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipientStanzaType::P256Tag => write!(f, "p256tag"),
            RecipientStanzaType::PivP256 => write!(f, "piv-p256"),
        }
    }
}

impl RecipientStanzaType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "p256tag" => Some(RecipientStanzaType::P256Tag),
            "piv-p256" => Some(RecipientStanzaType::PivP256),
            _ => None,
        }
    }
}

/// Stanza type for plugin communication
pub struct Stanza {
    pub type_str: String,
    pub args: Vec<String>,
    pub body: Vec<u8>,
}

impl Stanza {
    pub fn new<S: Into<String>>(type_str: S, args: Vec<String>, body: Vec<u8>) -> Self {
        Stanza {
            type_str: type_str.into(),
            args,
            body,
        }
    }
    
    pub fn as_error<S: Into<String>>(error_type: S, args: Vec<String>, message: &str) -> Self {
        Stanza {
            type_str: format!("error-{}", error_type.into()),
            args,
            body: message.as_bytes().to_vec(),
        }
    }
}

/// Main plugin implementation
pub struct Plugin<T: Crypto> {
    crypto: T,
}

impl<T: Crypto> Plugin<T> {
    pub fn new(crypto: T) -> Self {
        Plugin { crypto }
    }
    
    pub fn generate_key(&self, access_control: KeyAccessControl, recipient_type: RecipientType, now: DateTime<Utc>) -> Result<(String, String), PluginError> {
        if !self.crypto.is_secure_enclave_available() {
            return Err(PluginError::SEUnsupported);
        }
        
        // Create a new private key in the Secure Enclave
        let private_key = self.crypto.new_secure_enclave_private_key(None, Some(access_control))?;
        
        // Get the public key from the private key
        let public_key = private_key.public_key();
        
        // Format the identity and recipient strings
        let identity = age_identity_from_se_key(&*private_key)?;
        let recipient = age_recipient_from_public_key(&public_key, recipient_type)?;
        
        // Format the created timestamp
        let created_at = now.to_rfc3339();
        
        // Convert access control to string
        let access_control_str = match access_control {
            KeyAccessControl::None => "none",
            KeyAccessControl::Passcode => "passcode",
            KeyAccessControl::AnyBiometry => "any biometry",
            KeyAccessControl::AnyBiometryOrPasscode => "any biometry or passcode",
            KeyAccessControl::AnyBiometryAndPasscode => "any biometry and passcode",
            KeyAccessControl::CurrentBiometry => "current biometry",
            KeyAccessControl::CurrentBiometryAndPasscode => "current biometry and passcode",
        };
        
        // Format the output
        let contents = format!(
            "# created: {}\n# access control: {}\n# public key: {}\n{}\n\n",
            created_at, access_control_str, recipient, identity
        );
        
        Ok((contents, recipient))
    }
    
    pub fn generate_recipients(&self, input: &str, recipient_type: RecipientType) -> Result<String, PluginError> {
        // Placeholder - this would parse identity files and extract recipients
        // For now we just return an empty string
        Ok(String::new())
    }
    
    pub fn run_recipient_v1<S: Stream>(&mut self, stream: &mut S) {
        let mut recipients: Vec<String> = Vec::new();
        let mut identities: Vec<String> = Vec::new();
        let mut file_keys: Vec<Vec<u8>> = Vec::new();
        
        // Phase 1: Read stanzas
        loop {
            match read_stanza(stream) {
                Ok(stanza) => {
                    match stanza.type_str.as_str() {
                        "add-recipient" => {
                            if !stanza.args.is_empty() {
                                recipients.push(stanza.args[0].clone());
                            }
                        }
                        "add-identity" => {
                            if !stanza.args.is_empty() {
                                identities.push(stanza.args[0].clone());
                            }
                        }
                        "wrap-file-key" => {
                            file_keys.push(stanza.body.clone());
                        }
                        "done" => break,
                        _ => continue,
                    }
                }
                Err(_) => break,
            }
        }
        
        // Phase 2: Process stanzas
        // In a complete implementation, this would:
        // 1. Convert recipients to public keys
        // 2. Load identities from Secure Enclave
        // 3. Use each public key to encrypt each file key
        // 4. Send back the encrypted stanzas
        
        // For now, we just respond with a "done" stanza
        write_stanza(stream, Stanza::new("done", vec![], vec![]));
    }
    
    pub fn run_identity_v1<S: Stream>(&mut self, stream: &mut S) {
        // This is a placeholder for the identity operation
        // In a complete implementation, this would:
        // 1. Read identity and recipient stanzas
        // 2. Try to decrypt file keys using secure enclave identities
        // 3. Return decrypted file keys
        
        // For now, we just respond with a "done" stanza
        write_stanza(stream, Stanza::new("done", vec![], vec![]));
    }
}

// Converts a public key to an age recipient string with the given recipient type
fn age_recipient_from_public_key(public_key: &PublicKey, recipient_type: RecipientType) -> Result<String, PluginError> {
    let hrp = match recipient_type {
        RecipientType::SE => "age1se",
        RecipientType::P256Tag => "age1p256tag",
    };
    
    // Get compressed encoding of the public key
    let point = EncodedPoint::from(public_key);
    let compressed = point.compress().as_bytes().to_vec();
    
    let encoded = encode(hrp, compressed.to_base32(), Variant::Bech32)?;
    Ok(encoded)
}

// Converts a Secure Enclave private key to an age identity string
fn age_identity_from_se_key(private_key: &dyn SecureEnclavePrivateKey) -> Result<String, PluginError> {
    let data = private_key.data_representation();
    let encoded = encode("AGE-PLUGIN-SE-", data.to_base32(), Variant::Bech32)?;
    Ok(encoded)
}

fn read_stanza<S: Stream>(stream: &mut S) -> Result<Stanza, PluginError> {
    let header = match stream.read_line() {
        Some(line) => line,
        None => return Err(PluginError::IncompleteStanza),
    };
    
    let header_parts: Vec<&str> = header.split(' ').collect();
    if header_parts.is_empty() {
        return Err(PluginError::InvalidStanza);
    }
    
    let type_str = header_parts[0].to_string();
    let args = header_parts[1..].iter().map(|s| s.to_string()).collect();
    
    let mut body = Vec::new();
    loop {
        match stream.read_line() {
            Some(line) if line.is_empty() => break,
            Some(line) => {
                if let Ok(decoded) = BASE64.decode(line.as_bytes()) {
                    body.extend_from_slice(&decoded);
                } else {
                    return Err(PluginError::Base64Error);
                }
            }
            None => return Err(PluginError::IncompleteStanza),
        }
    }
    
    Ok(Stanza { type_str, args, body })
}

fn write_stanza<S: Stream>(stream: &mut S, stanza: Stanza) {
    let mut header = stanza.type_str;
    for arg in stanza.args {
        header.push(' ');
        header.push_str(&arg);
    }
    stream.write_line(&header);
    
    for chunk in stanza.body.chunks(64) {
        let encoded = BASE64.encode(chunk);
        stream.write_line(&encoded);
    }
    
    stream.write_line("");
} 