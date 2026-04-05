//! Age stanza parsing and serialization.
//!
//! Stanzas are the basic unit of the age plugin protocol, with format:
//! ```text
//! -> type arg1 arg2 ...
//! <base64-body-line-1>
//! <base64-body-line-2>
//! ...
//! <final-line (shorter than 64 chars)>
//! ```

use crate::encoding::base64::{decode_raw, encode_raw};
use crate::error::{Error, Result};

/// Maximum line length for base64 body chunks.
const MAX_LINE_LENGTH: usize = 64;

/// Stanza types supported by age-plugin-se.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StanzaType {
    /// The p256tag stanza type (preferred for Secure Enclave).
    P256Tag,
    /// The piv-p256 stanza type (for PIV compatibility).
    PivP256,
}

#[allow(clippy::should_implement_trait, clippy::missing_const_for_fn)]
impl StanzaType {
    /// Get the stanza type string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::P256Tag => "p256tag",
            Self::PivP256 => "piv-p256",
        }
    }

    /// Parse a stanza type from a string.
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "p256tag" => Some(Self::P256Tag),
            "piv-p256" => Some(Self::PivP256),
            _ => None,
        }
    }
}

/// A parsed age stanza.
#[derive(Debug, Clone)]
pub struct Stanza {
    /// The stanza type.
    pub stanza_type: StanzaType,
    /// The stanza arguments.
    pub args: Vec<String>,
    /// The decoded body.
    pub body: Vec<u8>,
}

#[allow(clippy::missing_const_for_fn)]
impl Stanza {
    /// Create a new stanza.
    #[must_use]
    pub fn new(stanza_type: StanzaType, args: Vec<String>, body: Vec<u8>) -> Self {
        Self {
            stanza_type,
            args,
            body,
        }
    }

    /// Parse a stanza from lines.
    ///
    /// The first line should be the header (-> type args...).
    /// Subsequent lines are the base64-encoded body.
    ///
    /// # Errors
    ///
    /// Returns an error if the stanza is malformed.
    pub fn parse(lines: &[String]) -> Result<Self> {
        if lines.is_empty() {
            return Err(Error::InvalidStanza("empty stanza".to_string()));
        }

        // Parse header line
        let header = &lines[0];
        let header = header
            .strip_prefix("-> ")
            .ok_or_else(|| Error::InvalidStanza("missing '-> ' prefix".to_string()))?;

        let mut parts = header.split_whitespace();
        let type_str = parts
            .next()
            .ok_or_else(|| Error::InvalidStanza("missing stanza type".to_string()))?;

        let stanza_type = StanzaType::from_str(type_str)
            .ok_or_else(|| Error::UnsupportedStanzaType(type_str.to_string()))?;

        let args: Vec<String> = parts.map(String::from).collect();

        // Parse body (remaining lines concatenated)
        let body_base64: String = lines[1..].iter().flat_map(|s| s.chars()).collect();
        let body = decode_raw(&body_base64)?;

        Ok(Self {
            stanza_type,
            args,
            body,
        })
    }

    /// Serialize the stanza to lines.
    ///
    /// Returns a vector of lines (without trailing newlines).
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Header line
        let mut header = format!("-> {}", self.stanza_type.as_str());
        for arg in &self.args {
            header.push(' ');
            header.push_str(arg);
        }
        lines.push(header);

        // Body lines (chunked at 64 characters)
        let encoded = encode_raw(&self.body);
        for chunk in encoded.as_bytes().chunks(MAX_LINE_LENGTH) {
            lines.push(String::from_utf8_lossy(chunk).to_string());
        }

        // Ensure there's at least one body line (even if empty)
        if self.body.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Write the stanza to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        for line in self.to_lines() {
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }
}

/// Parse a recipient stanza from input lines.
///
/// A recipient stanza looks like:
/// ```text
/// -> recipient-stanza 0 p256tag <tag-base64> <ephemeral-pubkey-base64>
/// <wrapped-file-key-base64>
/// ```
///
/// # Errors
///
/// Returns an error if the stanza is malformed.
pub fn parse_recipient_stanza(lines: &[String]) -> Result<(usize, Stanza)> {
    if lines.is_empty() {
        return Err(Error::InvalidStanza("empty recipient stanza".to_string()));
    }

    let header = &lines[0];
    let header = header
        .strip_prefix("-> recipient-stanza ")
        .ok_or_else(|| Error::InvalidStanza("not a recipient-stanza".to_string()))?;

    let mut parts = header.split_whitespace();

    // Parse file index
    let file_index: usize = parts
        .next()
        .ok_or_else(|| Error::InvalidStanza("missing file index".to_string()))?
        .parse()
        .map_err(|_| Error::InvalidStanza("invalid file index".to_string()))?;

    // Parse stanza type
    let type_str = parts
        .next()
        .ok_or_else(|| Error::InvalidStanza("missing stanza type".to_string()))?;

    let stanza_type = StanzaType::from_str(type_str)
        .ok_or_else(|| Error::UnsupportedStanzaType(type_str.to_string()))?;

    // Remaining parts are arguments
    let args: Vec<String> = parts.map(String::from).collect();

    // Parse body
    let body_base64: String = lines[1..].iter().flat_map(|s| s.chars()).collect();
    let body = decode_raw(&body_base64)?;

    Ok((
        file_index,
        Stanza {
            stanza_type,
            args,
            body,
        },
    ))
}

/// Create a recipient stanza for output.
///
/// # Arguments
///
/// * `file_index` - The file index (usually 0)
/// * `stanza_type` - The stanza type
/// * `tag` - The 4-byte recipient tag (base64 encoded as argument)
/// * `ephemeral_pubkey` - The ephemeral public key (base64 encoded as argument)
/// * `wrapped_key` - The wrapped file key (body)
#[must_use]
pub fn create_recipient_stanza(
    file_index: usize,
    stanza_type: StanzaType,
    tag: &[u8],
    ephemeral_pubkey: &[u8],
    wrapped_key: &[u8],
) -> Vec<String> {
    let mut lines = Vec::new();

    // Header: -> recipient-stanza <index> <type> <tag-b64> <ephemeral-b64>
    let header = format!(
        "-> recipient-stanza {} {} {} {}",
        file_index,
        stanza_type.as_str(),
        encode_raw(tag),
        encode_raw(ephemeral_pubkey)
    );
    lines.push(header);

    // Body: wrapped key (chunked)
    let encoded = encode_raw(wrapped_key);
    for chunk in encoded.as_bytes().chunks(MAX_LINE_LENGTH) {
        lines.push(String::from_utf8_lossy(chunk).to_string());
    }

    // Ensure at least one body line
    if wrapped_key.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_stanza_roundtrip() {
        let stanza = Stanza::new(
            StanzaType::P256Tag,
            vec!["arg1".to_string(), "arg2".to_string()],
            vec![0x01, 0x02, 0x03, 0x04],
        );

        let lines = stanza.to_lines();
        let parsed = Stanza::parse(&lines).unwrap();

        assert_eq!(parsed.stanza_type, stanza.stanza_type);
        assert_eq!(parsed.args, stanza.args);
        assert_eq!(parsed.body, stanza.body);
    }

    #[test]
    fn test_stanza_body_chunking() {
        // Create a body longer than 64 characters when base64 encoded
        let body = vec![0xAB; 100];
        let stanza = Stanza::new(StanzaType::PivP256, vec![], body.clone());

        let lines = stanza.to_lines();

        // First line is header
        assert!(lines[0].starts_with("-> piv-p256"));

        // Body should be split into multiple lines
        assert!(lines.len() > 2);

        // No line should exceed 64 characters (except possibly the header)
        for line in &lines[1..] {
            assert!(line.len() <= MAX_LINE_LENGTH);
        }

        // Roundtrip should work
        let parsed = Stanza::parse(&lines).unwrap();
        assert_eq!(parsed.body, body);
    }

    #[test]
    fn test_stanza_type_from_str() {
        assert_eq!(StanzaType::from_str("p256tag"), Some(StanzaType::P256Tag));
        assert_eq!(StanzaType::from_str("piv-p256"), Some(StanzaType::PivP256));
        assert_eq!(StanzaType::from_str("unknown"), None);
    }

    #[test]
    fn test_recipient_stanza_parsing() {
        let lines = vec![
            "-> recipient-stanza 0 p256tag AQID BAgM".to_string(),
            "EBQYHCAk".to_string(),
        ];

        let (index, stanza) = parse_recipient_stanza(&lines).unwrap();
        assert_eq!(index, 0);
        assert_eq!(stanza.stanza_type, StanzaType::P256Tag);
        assert_eq!(stanza.args.len(), 2);
    }

    #[test]
    fn test_create_recipient_stanza() {
        let lines = create_recipient_stanza(
            0,
            StanzaType::P256Tag,
            &[0x01, 0x02, 0x03, 0x04],
            &[0x02; 33],
            &[0xAB; 32],
        );

        assert!(lines[0].starts_with("-> recipient-stanza 0 p256tag"));
        assert!(lines.len() >= 2);
    }
}
