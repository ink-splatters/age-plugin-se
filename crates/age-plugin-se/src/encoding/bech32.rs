//! Bech32 encoding for age recipients and identities.
//!
//! age-plugin-se uses bech32 encoding with the following HRPs:
//! - `age1se`: Recipients (public keys)
//! - `AGE-PLUGIN-SE-`: Identities (key references)

use bech32::{Bech32, Bech32m, Hrp};

use crate::error::{Error, Result};

/// Human-readable part for age-plugin-se recipients.
pub const RECIPIENT_HRP: &str = "age1se";

/// Human-readable part for age-plugin-se identities (uppercase).
pub const IDENTITY_HRP: &str = "AGE-PLUGIN-SE-";

/// Encode data as bech32 with the given HRP.
///
/// # Errors
///
/// Returns an error if the HRP is invalid or encoding fails.
pub fn encode_bech32(hrp: &str, data: &[u8]) -> Result<String> {
    let hrp = Hrp::parse(hrp).map_err(|e| Error::Bech32(format!("invalid HRP '{hrp}': {e}")))?;

    bech32::encode::<Bech32>(hrp, data).map_err(|e| Error::Bech32(format!("encoding failed: {e}")))
}

/// Encode data as bech32m with the given HRP.
///
/// # Errors
///
/// Returns an error if the HRP is invalid or encoding fails.
pub fn encode_bech32m(hrp: &str, data: &[u8]) -> Result<String> {
    let hrp = Hrp::parse(hrp).map_err(|e| Error::Bech32(format!("invalid HRP '{hrp}': {e}")))?;

    bech32::encode::<Bech32m>(hrp, data).map_err(|e| Error::Bech32(format!("encoding failed: {e}")))
}

/// Decode bech32 data, returning the HRP and data.
///
/// # Errors
///
/// Returns an error if the input is not valid bech32.
pub fn decode_bech32(encoded: &str) -> Result<(String, Vec<u8>)> {
    // Try bech32 first, then bech32m
    bech32::decode(encoded)
        .map(|(hrp, data)| (hrp.to_string(), data))
        .map_err(|e| Error::Bech32(format!("decoding failed: {e}")))
}

/// Encode a recipient (public key) as bech32.
///
/// # Errors
///
/// Returns an error if encoding fails.
pub fn encode_recipient(public_key: &[u8]) -> Result<String> {
    encode_bech32(RECIPIENT_HRP, public_key)
}

/// Decode a recipient string.
///
/// # Errors
///
/// Returns an error if the recipient is not valid bech32 or has the wrong HRP.
pub fn decode_recipient(recipient: &str) -> Result<Vec<u8>> {
    let (hrp, data) = decode_bech32(recipient)?;

    if !hrp.eq_ignore_ascii_case(RECIPIENT_HRP) {
        return Err(Error::InvalidRecipient(format!(
            "expected HRP '{RECIPIENT_HRP}', got '{hrp}'"
        )));
    }

    Ok(data)
}

/// Encode an identity (key reference) as bech32.
///
/// Note: age identities use uppercase bech32 encoding.
///
/// # Errors
///
/// Returns an error if encoding fails.
pub fn encode_identity(key_data: &[u8]) -> Result<String> {
    encode_bech32(IDENTITY_HRP, key_data).map(|s| s.to_uppercase())
}

/// Decode an identity string.
///
/// # Errors
///
/// Returns an error if the identity is not valid bech32 or has the wrong HRP.
pub fn decode_identity(identity: &str) -> Result<Vec<u8>> {
    let (hrp, data) = decode_bech32(identity)?;

    if !hrp.eq_ignore_ascii_case(IDENTITY_HRP) {
        return Err(Error::InvalidIdentity(format!(
            "expected HRP '{IDENTITY_HRP}', got '{hrp}'"
        )));
    }

    Ok(data)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_recipient_roundtrip() {
        let public_key = vec![0x02; 33]; // Compressed P-256 public key size
        let encoded = encode_recipient(&public_key).unwrap();
        assert!(encoded.starts_with(RECIPIENT_HRP));

        let decoded = decode_recipient(&encoded).unwrap();
        assert_eq!(decoded, public_key);
    }

    #[test]
    fn test_identity_roundtrip() {
        let key_data = vec![0xAB; 32];
        let encoded = encode_identity(&key_data).unwrap();
        assert!(encoded.starts_with(IDENTITY_HRP));
        // Identity should be uppercase
        assert_eq!(encoded, encoded.to_uppercase());

        let decoded = decode_identity(&encoded).unwrap();
        assert_eq!(decoded, key_data);
    }

    #[test]
    fn test_decode_invalid_hrp() {
        let result = decode_recipient("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_decode() {
        let key_data = vec![0x42; 16];
        let encoded = encode_recipient(&key_data).unwrap();

        // Should work with uppercase too
        let decoded = decode_recipient(&encoded.to_uppercase()).unwrap();
        assert_eq!(decoded, key_data);
    }
}
