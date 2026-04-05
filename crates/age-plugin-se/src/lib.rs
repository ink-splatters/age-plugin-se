//! # age-plugin-se
//!
//! An age encryption plugin for Apple's Secure Enclave Processor.
//!
//! This plugin enables age encryption using P-256 keys stored in the Secure Enclave,
//! providing hardware-backed key protection with optional biometric authentication.
//!
//! ## Features
//!
//! - **Hardware-isolated keys**: Private keys never leave the Secure Enclave
//! - **Biometric protection**: Optional Touch ID/Face ID requirement for decryption
//! - **Device-bound keys**: Keys cannot be extracted or backed up
//! - **age compatibility**: Works with standard age tools
//!
//! ## Platform Support
//!
//! This plugin requires `macOS` with a Secure Enclave (M1+ Macs or Macs with T2 chip).
//! On other platforms, only encryption to existing recipients is supported.
//!
//! ## Usage
//!
//! ```text
//! # Generate a new key
//! age-plugin-se keygen
//!
//! # Encrypt a file
//! age -r age1se... -o secret.age plaintext.txt
//!
//! # Decrypt a file
//! age -d -i identity.txt -o plaintext.txt secret.age
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod crypto;
pub mod encoding;
pub mod error;
pub mod plugin;

pub use error::{Error, Result};
pub use plugin::recipient::{Identity, Recipient, RecipientType};
pub use plugin::stanza::{Stanza, StanzaType};

/// Re-export the Secure Enclave crate for direct access.
pub use apple_secure_enclave;

/// Check if the Secure Enclave is available.
///
/// This is a convenience wrapper around [`apple_secure_enclave::is_available`].
#[must_use]
pub fn is_secure_enclave_available() -> bool {
    apple_secure_enclave::is_available()
}

/// Generate a new Secure Enclave key with default access control.
///
/// This creates a key that requires device passcode or biometry for use.
///
/// # Errors
///
/// Returns an error if the Secure Enclave is not available or key generation fails.
#[cfg(target_os = "macos")]
pub fn generate_key() -> Result<(Identity, Recipient)> {
    use apple_secure_enclave::{AccessControl, SecureEnclaveKey};

    if !is_secure_enclave_available() {
        return Err(Error::SecureEnclaveNotAvailable);
    }

    // Create access control with passcode requirement
    let access_control = AccessControl::passcode()?;

    // Generate key
    let key = SecureEnclaveKey::generate(&access_control)?;

    // Get public key
    let public_key = key.public_key()?;
    let public_key_bytes = public_key.to_compressed_bytes()?;

    // Get key data for identity
    // Note: For SE keys, this may not be directly exportable
    // We store the public key as the key data for now
    let key_data = public_key_bytes.clone();

    let mut recipient_pubkey = [0u8; 33];
    recipient_pubkey.copy_from_slice(&public_key_bytes);

    let identity = Identity::with_public_key(key_data, recipient_pubkey);
    let recipient = Recipient::from_public_key(recipient_pubkey, RecipientType::P256Tag)?;

    Ok((identity, recipient))
}

/// Encrypt a file key to a recipient.
///
/// # Arguments
///
/// * `recipient` - The recipient to encrypt to
/// * `file_key` - The 16-byte file key to encrypt
///
/// # Returns
///
/// A tuple of (tag, `ephemeral_public_key`, `wrapped_file_key`).
///
/// # Errors
///
/// Returns an error if encryption fails.
pub fn encrypt_file_key(
    recipient: &Recipient,
    file_key: &[u8],
) -> Result<([u8; 4], [u8; 33], Vec<u8>)> {
    use crate::crypto::aead::wrap_file_key;
    use crate::crypto::kdf::{derive_wrap_key, recipient_tag};
    use crate::crypto::p256::p256_ecdh;

    // Perform ECDH
    let (shared_secret, ephemeral_public) = p256_ecdh(recipient.public_key())?;

    // Derive wrap key
    let wrap_key = derive_wrap_key(
        shared_secret.as_ref(),
        &ephemeral_public,
        recipient.public_key(),
    )?;

    // Calculate recipient tag
    let tag = recipient_tag(recipient.public_key(), &ephemeral_public)?;

    // Wrap file key
    let wrapped = wrap_file_key(&wrap_key, file_key)?;

    Ok((tag, ephemeral_public, wrapped))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_enclave_availability() {
        // This should not panic
        let _ = is_secure_enclave_available();
    }
}
