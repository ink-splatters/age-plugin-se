//! ChaCha20-Poly1305 AEAD for file key wrapping.
//!
//! The age protocol uses ChaCha20-Poly1305 with a zero nonce to wrap file keys.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// The size of a file key in bytes.
pub const FILE_KEY_SIZE: usize = 16;

/// The size of the AEAD tag in bytes.
pub const TAG_SIZE: usize = 16;

/// The size of the wrapped file key (file key + tag).
pub const WRAPPED_KEY_SIZE: usize = FILE_KEY_SIZE + TAG_SIZE;

/// Wrap a file key using ChaCha20-Poly1305.
///
/// # Arguments
///
/// * `wrap_key` - The 32-byte key derivation result
/// * `file_key` - The 16-byte file key to wrap
///
/// # Returns
///
/// The wrapped file key (32 bytes: 16 bytes ciphertext + 16 bytes tag).
///
/// # Errors
///
/// Returns an error if encryption fails.
pub fn wrap_file_key(wrap_key: &[u8], file_key: &[u8]) -> Result<Vec<u8>> {
    if wrap_key.len() != 32 {
        return Err(Error::Aead(format!(
            "wrap key must be 32 bytes, got {}",
            wrap_key.len()
        )));
    }

    if file_key.len() != FILE_KEY_SIZE {
        return Err(Error::Aead(format!(
            "file key must be {} bytes, got {}",
            FILE_KEY_SIZE,
            file_key.len()
        )));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(wrap_key)
        .map_err(|e| Error::Aead(format!("failed to create cipher: {e}")))?;

    // age uses a zero nonce for file key wrapping
    let nonce = Nonce::default();

    cipher
        .encrypt(&nonce, file_key)
        .map_err(|e| Error::Aead(format!("encryption failed: {e}")))
}

/// Unwrap a file key using ChaCha20-Poly1305.
///
/// # Arguments
///
/// * `wrap_key` - The 32-byte key derivation result
/// * `wrapped_key` - The 32-byte wrapped file key (ciphertext + tag)
///
/// # Returns
///
/// The unwrapped 16-byte file key.
///
/// # Errors
///
/// Returns an error if decryption or authentication fails.
pub fn unwrap_file_key(wrap_key: &[u8], wrapped_key: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if wrap_key.len() != 32 {
        return Err(Error::Aead(format!(
            "wrap key must be 32 bytes, got {}",
            wrap_key.len()
        )));
    }

    if wrapped_key.len() != WRAPPED_KEY_SIZE {
        return Err(Error::Aead(format!(
            "wrapped key must be {} bytes, got {}",
            WRAPPED_KEY_SIZE,
            wrapped_key.len()
        )));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(wrap_key)
        .map_err(|e| Error::Aead(format!("failed to create cipher: {e}")))?;

    // age uses a zero nonce for file key wrapping
    let nonce = Nonce::default();

    cipher
        .decrypt(&nonce, wrapped_key)
        .map(Zeroizing::new)
        .map_err(|e| Error::Aead(format!("decryption failed: {e}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let wrap_key = [0x42u8; 32];
        let file_key = [0xABu8; FILE_KEY_SIZE];

        let wrapped = wrap_file_key(&wrap_key, &file_key).unwrap();
        assert_eq!(wrapped.len(), WRAPPED_KEY_SIZE);

        let unwrapped = unwrap_file_key(&wrap_key, &wrapped).unwrap();
        assert_eq!(&unwrapped[..], &file_key);
    }

    #[test]
    fn test_invalid_wrap_key_size() {
        let wrap_key = [0x42u8; 16]; // Wrong size
        let file_key = [0xABu8; FILE_KEY_SIZE];

        let result = wrap_file_key(&wrap_key, &file_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_file_key_size() {
        let wrap_key = [0x42u8; 32];
        let file_key = [0xABu8; 8]; // Wrong size

        let result = wrap_file_key(&wrap_key, &file_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext() {
        let wrap_key = [0x42u8; 32];
        let file_key = [0xABu8; FILE_KEY_SIZE];

        let mut wrapped = wrap_file_key(&wrap_key, &file_key).unwrap();
        wrapped[0] ^= 0xFF; // Tamper with ciphertext

        let result = unwrap_file_key(&wrap_key, &wrapped);
        assert!(result.is_err());
    }
}
