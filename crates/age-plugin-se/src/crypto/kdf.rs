//! Key derivation functions for age-plugin-se.
//!
//! Uses HKDF-SHA256 for key derivation and HMAC-SHA256/SHA256 for recipient tags.

use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// The size of the recipient tag in bytes.
pub const TAG_SIZE: usize = 4;

/// The size of the derived wrap key in bytes.
pub const WRAP_KEY_SIZE: usize = 32;

/// Label for the piv-p256 stanza type.
const PIV_P256_LABEL: &[u8] = b"piv-p256";

/// Label for the p256tag stanza type.
const P256TAG_LABEL: &[u8] = b"p256tag";

/// Derive a wrap key from a shared secret using HKDF-SHA256.
///
/// # Arguments
///
/// * `shared_secret` - The ECDH shared secret
/// * `ephemeral_public_key` - The ephemeral public key (compressed, 33 bytes)
/// * `recipient_public_key` - The recipient's public key (compressed, 33 bytes)
///
/// # Returns
///
/// A 32-byte wrap key suitable for ChaCha20-Poly1305.
///
/// # Errors
///
/// Returns an error if key derivation fails.
pub fn derive_wrap_key(
    shared_secret: &[u8],
    ephemeral_public_key: &[u8],
    recipient_public_key: &[u8],
) -> Result<[u8; WRAP_KEY_SIZE]> {
    // Construct the info parameter: ephemeral_pub || recipient_pub
    let mut info = Vec::with_capacity(ephemeral_public_key.len() + recipient_public_key.len());
    info.extend_from_slice(ephemeral_public_key);
    info.extend_from_slice(recipient_public_key);

    // Use the label as salt for HKDF
    let hkdf = Hkdf::<Sha256>::new(Some(P256TAG_LABEL), shared_secret);

    let mut wrap_key = [0u8; WRAP_KEY_SIZE];
    hkdf.expand(&info, &mut wrap_key)
        .map_err(|e| Error::KeyDerivation(format!("HKDF expand failed: {e}")))?;

    Ok(wrap_key)
}

/// Derive a wrap key using the piv-p256 method.
///
/// This uses a different salt construction for compatibility with PIV-style recipients.
///
/// # Arguments
///
/// * `shared_secret` - The ECDH shared secret
/// * `ephemeral_public_key` - The ephemeral public key (uncompressed, 65 bytes)
/// * `recipient_public_key` - The recipient's public key (uncompressed, 65 bytes)
///
/// # Returns
///
/// A 32-byte wrap key suitable for ChaCha20-Poly1305.
///
/// # Errors
///
/// Returns an error if key derivation fails.
pub fn derive_wrap_key_piv(
    shared_secret: &[u8],
    ephemeral_public_key: &[u8],
    recipient_public_key: &[u8],
) -> Result<[u8; WRAP_KEY_SIZE]> {
    // For piv-p256, construct salt from the public keys
    let mut salt = Vec::with_capacity(ephemeral_public_key.len() + recipient_public_key.len());
    salt.extend_from_slice(ephemeral_public_key);
    salt.extend_from_slice(recipient_public_key);

    let hkdf = Hkdf::<Sha256>::new(Some(&salt), shared_secret);

    let mut wrap_key = [0u8; WRAP_KEY_SIZE];
    hkdf.expand(PIV_P256_LABEL, &mut wrap_key)
        .map_err(|e| Error::KeyDerivation(format!("HKDF expand failed: {e}")))?;

    Ok(wrap_key)
}

/// Calculate a 4-byte recipient tag using HMAC-SHA256.
///
/// The tag is used to identify which recipient can decrypt a stanza
/// without revealing the full public key.
///
/// # Arguments
///
/// * `recipient_public_key` - The recipient's public key (compressed, 33 bytes)
/// * `ephemeral_public_key` - The ephemeral public key (compressed, 33 bytes)
///
/// # Returns
///
/// A 4-byte tag.
///
/// # Errors
///
/// Returns an error if the key size is invalid (though this should never happen
/// for P-256 keys which are 33 bytes).
pub fn recipient_tag(
    recipient_public_key: &[u8],
    ephemeral_public_key: &[u8],
) -> Result<[u8; TAG_SIZE]> {
    // HMAC-SHA256(recipient_pub, ephemeral_pub)[0:4]
    let mut hmac = Hmac::<Sha256>::new_from_slice(recipient_public_key)
        .map_err(|e| Error::KeyDerivation(format!("HMAC initialization failed: {e}")))?;
    hmac.update(ephemeral_public_key);

    let result = hmac.finalize().into_bytes();
    let mut tag = [0u8; TAG_SIZE];
    tag.copy_from_slice(&result[..TAG_SIZE]);
    Ok(tag)
}

/// Calculate a 4-byte recipient tag using SHA256.
///
/// This is the piv-p256 variant that uses plain SHA256 instead of HMAC.
///
/// # Arguments
///
/// * `ephemeral_public_key` - The ephemeral public key (uncompressed, 65 bytes)
/// * `recipient_public_key` - The recipient's public key (uncompressed, 65 bytes)
///
/// # Returns
///
/// A 4-byte tag.
#[must_use]
pub fn recipient_tag_sha256(
    ephemeral_public_key: &[u8],
    recipient_public_key: &[u8],
) -> [u8; TAG_SIZE] {
    // SHA256(ephemeral_pub || recipient_pub)[0:4]
    let mut hasher = Sha256::new();
    hasher.update(ephemeral_public_key);
    hasher.update(recipient_public_key);

    let result = hasher.finalize();
    let mut tag = [0u8; TAG_SIZE];
    tag.copy_from_slice(&result[..TAG_SIZE]);
    tag
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_wrap_key_deterministic() {
        let shared_secret = [0x42u8; 32];
        let ephemeral_pub = [0x02u8; 33];
        let recipient_pub = [0x03u8; 33];

        let key1 = derive_wrap_key(&shared_secret, &ephemeral_pub, &recipient_pub).unwrap();
        let key2 = derive_wrap_key(&shared_secret, &ephemeral_pub, &recipient_pub).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_wrap_key_different_inputs() {
        let shared_secret = [0x42u8; 32];
        let ephemeral_pub = [0x02u8; 33];
        let recipient_pub1 = [0x03u8; 33];
        let recipient_pub2 = [0x04u8; 33];

        let key1 = derive_wrap_key(&shared_secret, &ephemeral_pub, &recipient_pub1).unwrap();
        let key2 = derive_wrap_key(&shared_secret, &ephemeral_pub, &recipient_pub2).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_recipient_tag_length() {
        let recipient_pub = [0x02u8; 33];
        let ephemeral_pub = [0x03u8; 33];

        let tag = recipient_tag(&recipient_pub, &ephemeral_pub).unwrap();
        assert_eq!(tag.len(), TAG_SIZE);
    }

    #[test]
    fn test_recipient_tag_deterministic() {
        let recipient_pub = [0x02u8; 33];
        let ephemeral_pub = [0x03u8; 33];

        let tag1 = recipient_tag(&recipient_pub, &ephemeral_pub).unwrap();
        let tag2 = recipient_tag(&recipient_pub, &ephemeral_pub).unwrap();

        assert_eq!(tag1, tag2);
    }

    #[test]
    fn test_recipient_tag_sha256_different() {
        // The two tag methods should produce different results
        let pub1 = [0x04u8; 65];
        let pub2 = [0x04u8; 65];

        let hmac_tag = recipient_tag(&pub1[..33], &pub2[..33]).unwrap();
        let sha_tag = recipient_tag_sha256(&pub1, &pub2);

        // They should be different (different algorithms)
        assert_ne!(hmac_tag, sha_tag);
    }
}
