//! # Apple Secure Enclave
//!
//! Safe Rust bindings for Apple's Secure Enclave Processor (SEP).
//!
//! The Secure Enclave is a hardware security module embedded in Apple devices
//! (A7+ chips, T2, M1+) that provides:
//!
//! - Hardware-isolated P256 key generation
//! - Key agreement (ECDH) without exposing private keys
//! - Biometric/passcode-gated access control
//! - Device-bound keys (non-exportable)
//!
//! ## Platform Support
//!
//! This crate only provides functionality on macOS. On other platforms,
//! all operations will return [`Error::NotAvailable`].
//!
//! ## Example
//!
//! ```no_run
//! use apple_secure_enclave::{AccessControl, SecureEnclaveKey};
//!
//! // Check if Secure Enclave is available
//! if !apple_secure_enclave::is_available() {
//!     eprintln!("Secure Enclave not available on this device");
//!     return Ok(());
//! }
//!
//! // Create access control requiring biometry
//! let access_control = AccessControl::biometry_any()?;
//!
//! // Generate a key in the Secure Enclave
//! let key = SecureEnclaveKey::generate(&access_control)?;
//!
//! // Export the public key (private key never leaves the enclave)
//! let public_key = key.public_key()?;
//! let public_key_bytes = public_key.to_compressed_bytes()?;
//! # Ok::<(), apple_secure_enclave::Error>(())
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod error;

#[cfg(target_os = "macos")]
mod ffi;

#[cfg(target_os = "macos")]
mod access_control;

#[cfg(target_os = "macos")]
mod key;

// Note: LAContext/AuthContext support removed to avoid ObjC runtime dependency.
// The Security framework will still prompt for biometry/passcode as needed.
// Custom prompt strings are not supported in this version.

pub use error::Error;

#[cfg(target_os = "macos")]
pub use access_control::{AccessControl, AccessControlFlags, Protection};

#[cfg(target_os = "macos")]
pub use key::{PublicKey, SecureEnclaveKey, SharedSecret};

/// Result type for Secure Enclave operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Check if the Secure Enclave is available on this device.
///
/// Returns `true` if the device has a Secure Enclave and it is accessible.
/// On non-`macOS` platforms, this always returns `false`.
///
/// # Example
///
/// ```
/// if apple_secure_enclave::is_available() {
///     println!("Secure Enclave is available");
/// } else {
///     println!("Secure Enclave is not available");
/// }
/// ```
#[must_use]
pub fn is_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        ffi::is_secure_enclave_available()
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(not(target_os = "macos"))]
mod stub {
    //! Stub implementations for non-`macOS` platforms.

    use super::*;

    /// Access control configuration (stub for non-`macOS`).
    #[derive(Debug, Clone)]
    pub struct AccessControl {
        _private: (),
    }

    impl AccessControl {
        /// Create access control with biometry requirement.
        ///
        /// # Errors
        ///
        /// Always returns [`Error::NotAvailable`] on non-`macOS` platforms.
        pub fn biometry_any() -> Result<Self> {
            Err(Error::NotAvailable)
        }

        /// Create access control with passcode requirement.
        ///
        /// # Errors
        ///
        /// Always returns [`Error::NotAvailable`] on non-`macOS` platforms.
        pub fn passcode() -> Result<Self> {
            Err(Error::NotAvailable)
        }
    }

    /// Secure Enclave key (stub for non-`macOS`).
    #[derive(Debug)]
    pub struct SecureEnclaveKey {
        _private: (),
    }

    impl SecureEnclaveKey {
        /// Generate a new key in the Secure Enclave.
        ///
        /// # Errors
        ///
        /// Always returns [`Error::NotAvailable`] on non-`macOS` platforms.
        pub fn generate(_access_control: &AccessControl) -> Result<Self> {
            Err(Error::NotAvailable)
        }
    }

    /// Public key (stub for non-`macOS`).
    #[derive(Debug, Clone)]
    pub struct PublicKey {
        _private: (),
    }

    /// Shared secret (stub for non-`macOS`).
    #[derive(Debug)]
    pub struct SharedSecret {
        _private: (),
    }
}

#[cfg(not(target_os = "macos"))]
pub use stub::{AccessControl, PublicKey, SecureEnclaveKey, SharedSecret};
