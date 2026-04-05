//! Raw base64 encoding/decoding (no padding).
//!
//! The age protocol uses base64 without padding for stanza bodies.

use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;

use crate::error::{Error, Result};

/// Encode bytes to raw base64 (no padding).
#[must_use]
pub fn encode_raw(data: &[u8]) -> String {
    STANDARD_NO_PAD.encode(data)
}

/// Decode raw base64 (no padding) to bytes.
///
/// # Errors
///
/// Returns an error if the input is not valid base64.
pub fn decode_raw(encoded: &str) -> Result<Vec<u8>> {
    // Try decoding with the standard no-pad engine first
    STANDARD_NO_PAD
        .decode(encoded)
        .or_else(|_| {
            // Fall back to trying with padding tolerance
            base64::engine::general_purpose::STANDARD.decode(encoded)
        })
        .map_err(Error::Base64)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = b"hello world";
        let encoded = encode_raw(original);
        let decoded = decode_raw(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_no_padding() {
        // "hello" encodes to "aGVsbG8" without padding
        let encoded = encode_raw(b"hello");
        assert!(!encoded.ends_with('='));
        assert_eq!(encoded, "aGVsbG8");
    }

    #[test]
    fn test_decode_with_padding_tolerance() {
        // Should decode correctly even with padding
        let decoded = decode_raw("aGVsbG8=").unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn test_empty() {
        let encoded = encode_raw(b"");
        assert_eq!(encoded, "");
        let decoded = decode_raw("").unwrap();
        assert_eq!(decoded, b"");
    }
}
