use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;
use chrono::Utc;
use crate::crypto::{KeyAccessControl, create_crypto};
use crate::plugin::{Plugin, RecipientType, Stream};
use std::fs;
use std::io::{self, Read, Write, BufRead};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Plugin error: {0}")]
    PluginError(#[from] crate::plugin::PluginError),

    #[error("Invalid command")]
    InvalidCommand,
}

#[derive(Parser)]
#[command(name = "rage-plugin-se")]
#[command(about = "Secure Enclave plugin for rage/age encryption", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new private key bound to the Secure Enclave
    Keygen(KeygenArgs),
    
    /// Read identities from a file and output corresponding recipients
    Recipients(FileArgs),
    
    /// Run in plugin mode (handled by age/rage)
    #[command(hide = true)]
    Plugin {
        #[arg(long = "age-plugin")]
        mode: PluginMode,
    },
}

#[derive(Args)]
struct KeygenArgs {
    /// Write the result to the file at path OUTPUT
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Access control for using the generated key
    #[arg(long = "access-control", value_enum, default_value_t = AccessControl::AnyBiometryOrPasscode)]
    access_control: AccessControl,
    
    /// Type of recipient to generate
    #[arg(long = "recipient-type", value_enum, default_value_t = RecipientTypeArg::Se)]
    recipient_type: RecipientTypeArg,
}

#[derive(Args)]
struct FileArgs {
    /// Write the result to the file at path OUTPUT
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Read data from the file at path INPUT
    #[arg(short, long)]
    input: Option<PathBuf>,
    
    /// Type of recipient to generate
    #[arg(long = "recipient-type", value_enum, default_value_t = RecipientTypeArg::Se)]
    recipient_type: RecipientTypeArg,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum AccessControl {
    #[value(name = "none")]
    None,
    
    #[value(name = "passcode")]
    Passcode,
    
    #[value(name = "any-biometry")]
    AnyBiometry,
    
    #[value(name = "any-biometry-or-passcode")]
    AnyBiometryOrPasscode,
    
    #[value(name = "any-biometry-and-passcode")]
    AnyBiometryAndPasscode,
    
    #[value(name = "current-biometry")]
    CurrentBiometry,
    
    #[value(name = "current-biometry-and-passcode")]
    CurrentBiometryAndPasscode,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum RecipientTypeArg {
    #[value(name = "se")]
    Se,
    
    #[value(name = "p256tag")]
    P256tag,
}

#[derive(Clone, Copy, Debug)]
enum PluginMode {
    RecipientV1,
    IdentityV1,
}

impl std::str::FromStr for PluginMode {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recipient-v1" => Ok(PluginMode::RecipientV1),
            "identity-v1" => Ok(PluginMode::IdentityV1),
            _ => Err(format!("Unknown plugin mode: {}", s)),
        }
    }
}

impl AccessControl {
    fn to_key_access_control(&self) -> KeyAccessControl {
        match self {
            AccessControl::None => KeyAccessControl::None,
            AccessControl::Passcode => KeyAccessControl::Passcode,
            AccessControl::AnyBiometry => KeyAccessControl::AnyBiometry,
            AccessControl::AnyBiometryOrPasscode => KeyAccessControl::AnyBiometryOrPasscode,
            AccessControl::AnyBiometryAndPasscode => KeyAccessControl::AnyBiometryAndPasscode,
            AccessControl::CurrentBiometry => KeyAccessControl::CurrentBiometry,
            AccessControl::CurrentBiometryAndPasscode => KeyAccessControl::CurrentBiometryAndPasscode,
        }
    }
}

impl RecipientTypeArg {
    fn to_recipient_type(&self) -> RecipientType {
        match self {
            RecipientTypeArg::Se => RecipientType::SE,
            RecipientTypeArg::P256tag => RecipientType::P256Tag,
        }
    }
}

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let crypto = create_crypto();
    
    match cli.command {
        Commands::Keygen(args) => {
            // Create a plugin without IO
            let plugin = Plugin::new(crypto);
            
            // Generate key with the given access control
            let (key_data, public_key) = plugin.generate_key(
                args.access_control.to_key_access_control(),
                args.recipient_type.to_recipient_type(),
                Utc::now()
            )?;
            
            // Write key to output or stdout
            if let Some(path) = args.output {
                fs::write(&path, key_data)?;
                // Set permissions to 600 (read/write for owner only)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(&path)?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    fs::set_permissions(&path, perms)?;
                }
                println!("Public key: {}", public_key);
            } else {
                print!("{}", key_data);
            }
        },
        
        Commands::Recipients(args) => {
            let plugin = Plugin::new(crypto);
            
            // Read input from file or stdin
            let input = if let Some(path) = args.input {
                fs::read_to_string(path)?
            } else {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            };
            
            // Generate recipients
            let recipients = plugin.generate_recipients(
                &input,
                args.recipient_type.to_recipient_type()
            )?;
            
            // Write to output or stdout
            if let Some(path) = args.output {
                fs::write(path, &recipients)?;
            } else if !recipients.is_empty() {
                println!("{}", recipients);
            }
        },
        
        Commands::Plugin { mode } => {
            let crypto = create_crypto();
            let stdin = io::stdin();
            let stdout = io::stdout();
            
            // Create stdio stream adapter
            let mut io = StdioStream {
                stdin: stdin.lock(),
                stdout: stdout.lock(),
            };
            
            // Create plugin without IO
            let mut plugin = Plugin::new(crypto);
            
            match mode {
                PluginMode::RecipientV1 => plugin.run_recipient_v1(&mut io),
                PluginMode::IdentityV1 => plugin.run_identity_v1(&mut io),
            }
        },
    }
    
    Ok(())
}

/// A simple stream implementation using stdin/stdout without any macros
pub struct StdioStream<'a> {
    stdin: io::StdinLock<'a>,
    stdout: io::StdoutLock<'a>,
}

impl<'a> Stream for StdioStream<'a> {
    fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.stdin.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                // Trim the trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(line)
            }
            Err(_) => None,
        }
    }
    
    fn write_line(&mut self, line: &str) {
        let _ = self.stdout.write_all(line.as_bytes());
        let _ = self.stdout.write_all(b"\n");
        let _ = self.stdout.flush();
    }
} 