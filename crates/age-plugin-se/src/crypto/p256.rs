//! P-256 elliptic curve operations.
//!
//! Provides ephemeral key generation, ECDH key agreement, and point compression.

use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{EncodedPoint, PublicKey};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// Size of a compressed P-256 public key (02/03 || x).
pub const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;

/// Size of an uncompressed P-256 public key (04 || x || y).
pub const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65;

/// Size of an ECDH shared secret.
pub const SHARED_SECRET_SIZE: usize = 32;

/// An ephemeral P-256 keypair for ECDH.
pub struct EphemeralKeypair {
    secret: EphemeralSecret,
    public_key: PublicKey,
}

impl EphemeralKeypair {
    /// Get the public key in compressed format.
    #[must_use]
    pub fn public_key_compressed(&self) -> [u8; COMPRESSED_PUBLIC_KEY_SIZE] {
        let encoded = self.public_key.to_encoded_point(true);
        let bytes = encoded.as_bytes();
        let mut result = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
        result.copy_from_slice(bytes);
        result
    }

    /// Get the public key in uncompressed format.
    #[must_use]
    pub fn public_key_uncompressed(&self) -> [u8; UNCOMPRESSED_PUBLIC_KEY_SIZE] {
        let encoded = self.public_key.to_encoded_point(false);
        let bytes = encoded.as_bytes();
        let mut result = [0u8; UNCOMPRESSED_PUBLIC_KEY_SIZE];
        result.copy_from_slice(bytes);
        result
    }

    /// Perform ECDH key agreement with a peer's public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the peer's public key is invalid.
    pub fn diffie_hellman(
        self,
        peer_public_key: &[u8],
    ) -> Result<Zeroizing<[u8; SHARED_SECRET_SIZE]>> {
        let peer_key = parse_public_key(peer_public_key)?;

        let shared_secret = self.secret.diffie_hellman(&peer_key);
        let bytes = shared_secret.raw_secret_bytes();

        let mut result = Zeroizing::new([0u8; SHARED_SECRET_SIZE]);
        result.copy_from_slice(bytes);
        Ok(result)
    }
}

/// Generate a new ephemeral P-256 keypair.
///
/// The private key is randomly generated and suitable for one-time use in ECDH.
#[must_use]
pub fn generate_ephemeral_keypair() -> EphemeralKeypair {
    let secret = EphemeralSecret::random(&mut rand_core::OsRng);
    let public_key = secret.public_key();
    EphemeralKeypair { secret, public_key }
}

/// Perform ECDH key agreement between an ephemeral keypair and a peer's public key.
///
/// # Arguments
///
/// * `peer_public_key` - The peer's public key (compressed or uncompressed).
///
/// # Returns
///
/// A tuple of (`shared_secret`, `ephemeral_public_key_compressed`).
///
/// # Errors
///
/// Returns an error if the peer's public key is invalid.
pub fn p256_ecdh(
    peer_public_key: &[u8],
) -> Result<(
    Zeroizing<[u8; SHARED_SECRET_SIZE]>,
    [u8; COMPRESSED_PUBLIC_KEY_SIZE],
)> {
    let keypair = generate_ephemeral_keypair();
    let ephemeral_public = keypair.public_key_compressed();
    let shared_secret = keypair.diffie_hellman(peer_public_key)?;
    Ok((shared_secret, ephemeral_public))
}

/// Parse a public key from bytes (compressed or uncompressed).
fn parse_public_key(bytes: &[u8]) -> Result<PublicKey> {
    let encoded = EncodedPoint::from_bytes(bytes)
        .map_err(|e| Error::InvalidPublicKey(format!("invalid encoding: {e}")))?;

    Option::<PublicKey>::from(PublicKey::from_encoded_point(&encoded))
        .ok_or_else(|| Error::InvalidPublicKey("point not on curve".to_string()))
}

/// Compress a P-256 public key.
///
/// # Arguments
///
/// * `uncompressed` - The uncompressed public key (04 || x || y, 65 bytes).
///
/// # Returns
///
/// The compressed public key (02/03 || x, 33 bytes).
///
/// # Errors
///
/// Returns an error if the input is not a valid uncompressed public key.
pub fn compress_public_key(uncompressed: &[u8]) -> Result<[u8; COMPRESSED_PUBLIC_KEY_SIZE]> {
    if uncompressed.len() != UNCOMPRESSED_PUBLIC_KEY_SIZE {
        return Err(Error::InvalidPublicKey(format!(
            "expected {} bytes, got {}",
            UNCOMPRESSED_PUBLIC_KEY_SIZE,
            uncompressed.len()
        )));
    }

    let public_key = parse_public_key(uncompressed)?;
    let compressed = public_key.to_encoded_point(true);

    let mut result = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
    result.copy_from_slice(compressed.as_bytes());
    Ok(result)
}

/// Decompress a P-256 public key.
///
/// # Arguments
///
/// * `compressed` - The compressed public key (02/03 || x, 33 bytes).
///
/// # Returns
///
/// The uncompressed public key (04 || x || y, 65 bytes).
///
/// # Errors
///
/// Returns an error if the input is not a valid compressed public key.
pub fn decompress_public_key(compressed: &[u8]) -> Result<[u8; UNCOMPRESSED_PUBLIC_KEY_SIZE]> {
    if compressed.len() != COMPRESSED_PUBLIC_KEY_SIZE {
        return Err(Error::InvalidPublicKey(format!(
            "expected {} bytes, got {}",
            COMPRESSED_PUBLIC_KEY_SIZE,
            compressed.len()
        )));
    }

    let public_key = parse_public_key(compressed)?;
    let uncompressed = public_key.to_encoded_point(false);

    let mut result = [0u8; UNCOMPRESSED_PUBLIC_KEY_SIZE];
    result.copy_from_slice(uncompressed.as_bytes());
    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_keypair_generation() {
        let keypair = generate_ephemeral_keypair();
        let compressed = keypair.public_key_compressed();
        let uncompressed = keypair.public_key_uncompressed();

        assert_eq!(compressed.len(), COMPRESSED_PUBLIC_KEY_SIZE);
        assert_eq!(uncompressed.len(), UNCOMPRESSED_PUBLIC_KEY_SIZE);

        // Check prefixes
        assert!(compressed[0] == 0x02 || compressed[0] == 0x03);
        assert_eq!(uncompressed[0], 0x04);
    }

    #[test]
    fn test_ecdh_agreement() {
        let alice = generate_ephemeral_keypair();
        let bob = generate_ephemeral_keypair();

        let alice_public = alice.public_key_compressed();
        let bob_public = bob.public_key_compressed();

        let alice_secret = alice.diffie_hellman(&bob_public).unwrap();
        let bob_secret = bob.diffie_hellman(&alice_public).unwrap();

        assert_eq!(*alice_secret, *bob_secret);
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let keypair = generate_ephemeral_keypair();
        let uncompressed = keypair.public_key_uncompressed();
        let compressed = compress_public_key(&uncompressed).unwrap();
        let decompressed = decompress_public_key(&compressed).unwrap();

        assert_eq!(uncompressed, decompressed);
    }

    #[test]
    fn test_invalid_public_key() {
        let invalid = [0x04u8; 65]; // All zeros except prefix - not on curve
        let result = parse_public_key(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_compressed_format_parsing() {
        let keypair = generate_ephemeral_keypair();
        let compressed = keypair.public_key_compressed();

        // Should be able to parse compressed format
        let result = parse_public_key(&compressed);
        assert!(result.is_ok());
    }
}
