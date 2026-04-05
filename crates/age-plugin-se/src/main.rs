//! age-plugin-se - Age encryption plugin for Apple Secure Enclave.

use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};

use age_plugin_se::encoding::base64::{decode_raw, encode_raw};
use age_plugin_se::error::Result;
use age_plugin_se::plugin::stanza::{StanzaType, create_recipient_stanza, parse_recipient_stanza};
#[allow(unused_imports)]
use age_plugin_se::plugin::{is_grease, parse_command, write_done, write_fail};
use age_plugin_se::{Identity, Recipient, RecipientType, encrypt_file_key};

/// Age encryption plugin for Apple Secure Enclave.
#[derive(Parser)]
#[command(name = "age-plugin-se")]
#[command(about = "Age encryption plugin for Apple Secure Enclave")]
#[command(version)]
struct Cli {
    /// Run as age-plugin in state machine mode.
    #[arg(long, value_name = "STATE")]
    age_plugin: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Secure Enclave key.
    #[command(name = "keygen")]
    Keygen {
        /// Output format (default: age identity format).
        #[arg(short, long, default_value = "age")]
        format: String,
    },

    /// Print the recipient for an identity.
    #[command(name = "recipients")]
    Recipients,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(state) = cli.age_plugin {
        return run_plugin(&state);
    }

    match cli.command {
        Some(Commands::Keygen { format: _ }) => run_keygen(),
        Some(Commands::Recipients) => run_recipients(),
        None => {
            // No command specified, print help
            eprintln!("age-plugin-se: use --help for usage");
            Ok(())
        }
    }
}

/// Run the plugin in state machine mode.
fn run_plugin(state: &str) -> Result<()> {
    match state {
        "recipient-v1" => run_recipient_v1(),
        "identity-v1" => run_identity_v1(),
        _ => {
            eprintln!("unsupported plugin state: {state}");
            std::process::exit(1);
        }
    }
}

/// Run the recipient-v1 protocol (encryption).
#[allow(clippy::significant_drop_tightening)]
fn run_recipient_v1() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    let mut recipients: Vec<Recipient> = Vec::new();
    let mut file_keys: Vec<Vec<u8>> = Vec::new();

    // Phase 1: Read commands
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();

        if line == "done" {
            break;
        }

        if let Some((cmd, args)) = parse_command(line) {
            if is_grease(cmd) {
                // Ignore grease commands
                continue;
            }

            match cmd {
                "add-recipient" => {
                    if let Some(recipient_str) = args.first() {
                        match Recipient::from_string(recipient_str) {
                            Ok(recipient) => recipients.push(recipient),
                            Err(e) => {
                                writeln!(stdout, "-> error add-recipient")?;
                                writeln!(stdout, "{e}")?;
                                writeln!(stdout)?;
                            }
                        }
                    }
                }
                "wrap-file-key" => {
                    // Read the file key body
                    let mut body_lines = Vec::new();
                    loop {
                        let mut body_line = String::new();
                        reader.read_line(&mut body_line)?;
                        let body_line = body_line.trim();
                        if body_line.is_empty() {
                            break;
                        }
                        body_lines.push(body_line.to_string());
                    }

                    let body_b64: String = body_lines.concat();
                    if let Ok(file_key) = decode_raw(&body_b64) {
                        file_keys.push(file_key);
                    }
                }
                _ => {
                    // Unknown command, ignore
                }
            }
        }
    }

    // Phase 2: Emit recipient stanzas
    for (file_index, file_key) in file_keys.iter().enumerate() {
        for recipient in &recipients {
            match encrypt_file_key(recipient, file_key) {
                Ok((tag, ephemeral_pub, wrapped_key)) => {
                    let stanza_type = match recipient.recipient_type() {
                        RecipientType::P256Tag => StanzaType::P256Tag,
                        RecipientType::PivP256 => StanzaType::PivP256,
                    };

                    let lines = create_recipient_stanza(
                        file_index,
                        stanza_type,
                        &tag,
                        &ephemeral_pub,
                        &wrapped_key,
                    );

                    for line in lines {
                        writeln!(stdout, "{line}")?;
                    }
                }
                Err(e) => {
                    writeln!(stdout, "-> error recipient")?;
                    writeln!(stdout, "{e}")?;
                    writeln!(stdout)?;
                }
            }
        }
    }

    write_done(&mut stdout)?;
    Ok(())
}

/// Run the identity-v1 protocol (decryption).
#[allow(clippy::too_many_lines, clippy::significant_drop_tightening)]
fn run_identity_v1() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    let mut identities: Vec<Identity> = Vec::new();
    let mut stanzas: Vec<(usize, age_plugin_se::Stanza)> = Vec::new();

    // Phase 1: Read commands
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();

        if line == "done" {
            break;
        }

        if let Some((cmd, args)) = parse_command(line) {
            if is_grease(cmd) {
                continue;
            }

            match cmd {
                "add-identity" => {
                    if let Some(identity_str) = args.first() {
                        match Identity::from_string(identity_str) {
                            Ok(identity) => identities.push(identity),
                            Err(e) => {
                                writeln!(stdout, "-> error add-identity")?;
                                writeln!(stdout, "{e}")?;
                                writeln!(stdout)?;
                            }
                        }
                    }
                }
                "recipient-stanza" => {
                    // Parse the stanza
                    let header_line = format!("-> recipient-stanza {}", args.join(" "));
                    let mut stanza_lines = vec![header_line];

                    // Read body lines
                    loop {
                        let mut body_line = String::new();
                        reader.read_line(&mut body_line)?;
                        let body_line = body_line.trim();
                        if body_line.is_empty() {
                            break;
                        }
                        stanza_lines.push(body_line.to_string());
                    }

                    if let Ok((file_index, stanza)) = parse_recipient_stanza(&stanza_lines) {
                        stanzas.push((file_index, stanza));
                    }
                }
                _ => {}
            }
        }
    }

    // Phase 2: Try to decrypt
    // For now, we just report that we can't decrypt without Secure Enclave access
    // In a full implementation, we would use the Secure Enclave to perform ECDH

    #[cfg(target_os = "macos")]
    {
        use age_plugin_se::crypto::aead::unwrap_file_key;
        use age_plugin_se::crypto::kdf::{derive_wrap_key, recipient_tag};
        use apple_secure_enclave::SecureEnclaveKey;

        for (file_index, stanza) in &stanzas {
            // Try each identity
            for identity in &identities {
                // Try to restore the SE key
                if let Ok(se_key) = SecureEnclaveKey::from_data(identity.key_data()) {
                    // Get our public key
                    if let Ok(our_public) = se_key.public_key() {
                        if let Ok(our_public_bytes) = our_public.to_compressed_bytes() {
                            // Parse stanza arguments: tag and ephemeral public key
                            if stanza.args.len() >= 2 {
                                let tag_b64 = &stanza.args[0];
                                let ephemeral_b64 = &stanza.args[1];

                                if let (Ok(tag), Ok(ephemeral_pub)) =
                                    (decode_raw(tag_b64), decode_raw(ephemeral_b64))
                                {
                                    // Check if this stanza is for us
                                    if let Ok(expected_tag) =
                                        recipient_tag(&our_public_bytes, &ephemeral_pub)
                                    {
                                        if tag == expected_tag {
                                            // Create public key from ephemeral
                                            if let Ok(ephemeral_key) =
                                            apple_secure_enclave::PublicKey::from_compressed_bytes(
                                                &ephemeral_pub,
                                            )
                                        {
                                            // Perform ECDH in Secure Enclave
                                            if let Ok(shared_secret) =
                                                se_key.shared_secret(&ephemeral_key)
                                            {
                                                // Derive wrap key
                                                if let Ok(wrap_key) = derive_wrap_key(
                                                    shared_secret.as_bytes(),
                                                    &ephemeral_pub,
                                                    &our_public_bytes,
                                                ) {
                                                    // Unwrap file key
                                                    if let Ok(file_key) =
                                                        unwrap_file_key(&wrap_key, &stanza.body)
                                                    {
                                                        // Success! Output the file key
                                                        writeln!(
                                                            stdout,
                                                            "-> file-key {file_index}"
                                                        )?;
                                                        let encoded = encode_raw(&file_key);
                                                        writeln!(stdout, "{encoded}")?;
                                                        writeln!(stdout)?;
                                                    }
                                                }
                                            }
                                        }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS, we can't decrypt
        let _ = (identities, stanzas);
        write_fail(&mut stdout)?;
        return Ok(());
    }

    write_done(&mut stdout)?;
    Ok(())
}

/// Generate a new Secure Enclave key.
fn run_keygen() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let (identity, recipient) = age_plugin_se::generate_key()?;

        let identity_str = identity.to_string()?;
        let recipient_str = recipient.to_string()?;

        println!("# created: {}", chrono_lite::Utc::now());
        println!("# public key: {recipient_str}");
        println!("{identity_str}");

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("error: Secure Enclave is only available on macOS");
        std::process::exit(1);
    }
}

/// Print recipients for identities read from stdin.
fn run_recipients() -> Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Try to parse as identity
        if let Ok(identity) = Identity::from_string(line) {
            // If identity has a public key, output as recipient
            if let Some(public_key) = identity.public_key() {
                if let Ok(recipient) =
                    Recipient::from_public_key(*public_key, RecipientType::P256Tag)
                {
                    if let Ok(recipient_str) = recipient.to_string() {
                        println!("{recipient_str}");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Simple UTC timestamp helper (avoiding chrono dependency).
mod chrono_lite {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct Utc;

    impl Utc {
        pub fn now() -> String {
            let duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();

            // Simple ISO 8601 formatting
            let days_since_epoch = secs / 86400;
            let time_of_day = secs % 86400;

            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;

            // Calculate year, month, day (simplified - not handling leap years perfectly)
            let mut year = 1970;
            let mut remaining_days = days_since_epoch;

            loop {
                let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                if remaining_days < days_in_year {
                    break;
                }
                remaining_days -= days_in_year;
                year += 1;
            }

            let days_in_months: [u64; 12] = if is_leap_year(year) {
                [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            } else {
                [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            };

            let mut month = 1;
            for &days in &days_in_months {
                if remaining_days < days {
                    break;
                }
                remaining_days -= days;
                month += 1;
            }
            let day = remaining_days + 1;

            format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
        }
    }

    const fn is_leap_year(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}
