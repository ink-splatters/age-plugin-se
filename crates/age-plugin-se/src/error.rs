//! Error types for age-plugin-se.

use thiserror::Error;

/// Errors that can occur during plugin operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The Secure Enclave is not available on this platform.
    #[error("Secure Enclave not available")]
    SecureEnclaveNotAvailable,

    /// An error occurred in the Secure Enclave.
    #[error("Secure Enclave error: {0}")]
    SecureEnclave(#[from] apple_secure_enclave::Error),

    /// Invalid recipient format.
    #[error("invalid recipient: {0}")]
    InvalidRecipient(String),

    /// Invalid identity format.
    #[error("invalid identity: {0}")]
    InvalidIdentity(String),

    /// Invalid stanza format.
    #[error("invalid stanza: {0}")]
    InvalidStanza(String),

    /// Bech32 encoding/decoding error.
    #[error("bech32 error: {0}")]
    Bech32(String),

    /// Base64 encoding/decoding error.
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// AEAD encryption/decryption error.
    #[error("AEAD error: {0}")]
    Aead(String),

    /// Key derivation error.
    #[error("key derivation error: {0}")]
    KeyDerivation(String),

    /// Protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid public key data.
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    /// ECDH key agreement failed.
    #[error("key agreement failed: {0}")]
    KeyAgreement(String),

    /// Unsupported stanza type.
    #[error("unsupported stanza type: {0}")]
    UnsupportedStanzaType(String),
}

/// Result type for plugin operations.
pub type Result<T> = std::result::Result<T, Error>;
