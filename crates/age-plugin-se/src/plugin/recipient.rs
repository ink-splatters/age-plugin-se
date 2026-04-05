//! Recipient and identity types for age-plugin-se.

use crate::crypto::p256::COMPRESSED_PUBLIC_KEY_SIZE;
use crate::encoding::bech32::{
    decode_identity, decode_recipient, encode_identity, encode_recipient,
};
use crate::error::{Error, Result};

/// The type of recipient stanza to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecipientType {
    /// Use p256tag stanzas (recommended for Secure Enclave).
    #[default]
    P256Tag,
    /// Use piv-p256 stanzas (for PIV compatibility).
    PivP256,
}

/// A recipient (public key) that can encrypt to.
#[derive(Debug, Clone)]
pub struct Recipient {
    /// The compressed public key (33 bytes).
    public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE],
    /// The recipient type to use for encryption.
    recipient_type: RecipientType,
}

#[allow(clippy::should_implement_trait, clippy::missing_const_for_fn)]
impl Recipient {
    /// Create a recipient from a compressed public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the public key is invalid.
    pub fn from_public_key(
        public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE],
        recipient_type: RecipientType,
    ) -> Result<Self> {
        // Validate prefix
        if public_key[0] != 0x02 && public_key[0] != 0x03 {
            return Err(Error::InvalidPublicKey(
                "expected compressed point prefix (0x02 or 0x03)".to_string(),
            ));
        }

        Ok(Self {
            public_key,
            recipient_type,
        })
    }

    /// Parse a recipient from a bech32-encoded string.
    ///
    /// # Errors
    ///
    /// Returns an error if the recipient string is invalid.
    pub fn from_string(s: &str) -> Result<Self> {
        let data = decode_recipient(s)?;

        if data.len() != COMPRESSED_PUBLIC_KEY_SIZE {
            return Err(Error::InvalidRecipient(format!(
                "expected {} bytes, got {}",
                COMPRESSED_PUBLIC_KEY_SIZE,
                data.len()
            )));
        }

        let mut public_key = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
        public_key.copy_from_slice(&data);

        Self::from_public_key(public_key, RecipientType::P256Tag)
    }

    /// Encode the recipient as a bech32 string.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn to_string(&self) -> Result<String> {
        encode_recipient(&self.public_key)
    }

    /// Get the compressed public key.
    #[must_use]
    pub const fn public_key(&self) -> &[u8; COMPRESSED_PUBLIC_KEY_SIZE] {
        &self.public_key
    }

    /// Get the recipient type.
    #[must_use]
    pub const fn recipient_type(&self) -> RecipientType {
        self.recipient_type
    }

    /// Set the recipient type.
    pub fn set_recipient_type(&mut self, recipient_type: RecipientType) {
        self.recipient_type = recipient_type;
    }
}

/// An identity that can decrypt messages.
///
/// In age-plugin-se, the identity contains the key data needed to
/// restore the Secure Enclave key reference.
#[derive(Debug, Clone)]
#[allow(clippy::should_implement_trait)]
pub struct Identity {
    /// The key data (opaque reference to the Secure Enclave key).
    key_data: Vec<u8>,
    /// The public key (for matching stanzas).
    public_key: Option<[u8; COMPRESSED_PUBLIC_KEY_SIZE]>,
}

#[allow(clippy::missing_const_for_fn)]
impl Identity {
    /// Create an identity from key data.
    #[must_use]
    pub fn new(key_data: Vec<u8>) -> Self {
        Self {
            key_data,
            public_key: None,
        }
    }

    /// Create an identity with a known public key.
    #[must_use]
    pub fn with_public_key(
        key_data: Vec<u8>,
        public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE],
    ) -> Self {
        Self {
            key_data,
            public_key: Some(public_key),
        }
    }

    /// Parse an identity from a bech32-encoded string.
    ///
    /// # Errors
    ///
    /// Returns an error if the identity string is invalid.
    pub fn from_string(s: &str) -> Result<Self> {
        let key_data = decode_identity(s)?;
        Ok(Self::new(key_data))
    }

    /// Encode the identity as a bech32 string.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn to_string(&self) -> Result<String> {
        encode_identity(&self.key_data)
    }

    /// Get the key data.
    #[must_use]
    pub fn key_data(&self) -> &[u8] {
        &self.key_data
    }

    /// Get the public key, if known.
    #[must_use]
    pub const fn public_key(&self) -> Option<&[u8; COMPRESSED_PUBLIC_KEY_SIZE]> {
        self.public_key.as_ref()
    }

    /// Set the public key.
    pub fn set_public_key(&mut self, public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE]) {
        self.public_key = Some(public_key);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::cast_possible_truncation)]
mod tests {
    use super::*;

    #[test]
    fn test_recipient_roundtrip() {
        let mut public_key = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
        public_key[0] = 0x02; // Compressed point prefix
        for (i, byte) in public_key.iter_mut().enumerate().skip(1) {
            *byte = i as u8;
        }

        let recipient = Recipient::from_public_key(public_key, RecipientType::P256Tag).unwrap();
        let encoded = recipient.to_string().unwrap();
        let decoded = Recipient::from_string(&encoded).unwrap();

        assert_eq!(decoded.public_key(), recipient.public_key());
    }

    #[test]
    fn test_identity_roundtrip() {
        let key_data = vec![0xAB; 64];
        let identity = Identity::new(key_data.clone());

        let encoded = identity.to_string().unwrap();
        let decoded = Identity::from_string(&encoded).unwrap();

        assert_eq!(decoded.key_data(), &key_data);
    }

    #[test]
    fn test_invalid_recipient_prefix() {
        let mut public_key = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
        public_key[0] = 0x04; // Wrong prefix (uncompressed)

        let result = Recipient::from_public_key(public_key, RecipientType::P256Tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_recipient_type_default() {
        let recipient_type = RecipientType::default();
        assert_eq!(recipient_type, RecipientType::P256Tag);
    }
}
