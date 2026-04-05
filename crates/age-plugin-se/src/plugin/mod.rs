//! Age plugin protocol implementation.
//!
//! This module implements the age plugin protocol for recipient-v1 and identity-v1.

pub mod recipient;
pub mod stanza;

pub use recipient::{Recipient, RecipientType};
pub use stanza::{Stanza, StanzaType};

use std::io::{BufRead, Write};

use crate::error::{Error, Result};

/// Read lines from stdin until "done" is received.
///
/// # Errors
///
/// Returns an error if reading fails or the protocol is violated.
pub fn read_until_done<R: BufRead>(reader: &mut R) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Err(Error::Protocol("unexpected EOF".to_string()));
        }

        let trimmed = line.trim_end();
        if trimmed == "done" {
            break;
        }

        lines.push(trimmed.to_string());
    }

    Ok(lines)
}

/// Write an "ok" response.
///
/// # Errors
///
/// Returns an error if writing fails.
pub fn write_ok<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer, "ok")?;
    writer.flush()?;
    Ok(())
}

/// Write a "done" marker.
///
/// # Errors
///
/// Returns an error if writing fails.
pub fn write_done<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer, "done")?;
    writer.flush()?;
    Ok(())
}

/// Write a "fail" response.
///
/// # Errors
///
/// Returns an error if writing fails.
pub fn write_fail<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer, "fail")?;
    writer.flush()?;
    Ok(())
}

/// Parse a line as a command.
///
/// Commands have the format: `-> command arg1 arg2 ...`
#[must_use]
pub fn parse_command(line: &str) -> Option<(&str, Vec<&str>)> {
    let line = line.strip_prefix("-> ")?;
    let mut parts = line.split_whitespace();
    let command = parts.next()?;
    let args: Vec<&str> = parts.collect();
    Some((command, args))
}

/// Parse grease (ignored commands starting with "grease").
#[must_use]
pub fn is_grease(command: &str) -> bool {
    command.starts_with("grease")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_until_done() {
        let input = "-> add-recipient age1se...\n-> wrap-file-key\nbody\ndone\n";
        let mut reader = Cursor::new(input);

        let lines = read_until_done(&mut reader).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "-> add-recipient age1se...");
        assert_eq!(lines[1], "-> wrap-file-key");
        assert_eq!(lines[2], "body");
    }

    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("-> add-recipient age1se123").unwrap();
        assert_eq!(cmd, "add-recipient");
        assert_eq!(args, vec!["age1se123"]);
    }

    #[test]
    fn test_parse_command_no_args() {
        let (cmd, args) = parse_command("-> done").unwrap();
        assert_eq!(cmd, "done");
        assert!(args.is_empty());
    }

    #[test]
    fn test_is_grease() {
        assert!(is_grease("grease"));
        assert!(is_grease("grease-abc"));
        assert!(!is_grease("add-recipient"));
    }
}
