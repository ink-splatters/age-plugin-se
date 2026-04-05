//! Error types for Secure Enclave operations.

use std::fmt;

/// Errors that can occur during Secure Enclave operations.
#[derive(Debug)]
pub enum Error {
    /// Secure Enclave is not available on this device.
    ///
    /// This can occur because:
    /// - The device doesn't have a Secure Enclave (older Macs without T2/Apple Silicon)
    /// - Running on a non-macOS platform
    /// - The Secure Enclave is disabled or inaccessible
    NotAvailable,

    /// Failed to create access control configuration.
    AccessControlCreation(String),

    /// Failed to generate a key in the Secure Enclave.
    KeyGeneration(String),

    /// Failed to perform a key operation (e.g., ECDH, export).
    KeyOperation(String),

    /// The provided key data is invalid or malformed.
    InvalidKeyData(String),

    /// User authentication failed (e.g., biometry not recognized, passcode incorrect).
    AuthenticationFailed,

    /// Authentication was cancelled by the user.
    AuthenticationCancelled,

    /// The operation was denied due to access control restrictions.
    AccessDenied,

    /// An internal error occurred in the Security framework.
    SecurityFramework(i32, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAvailable => {
                write!(f, "Secure Enclave is not available on this device")
            }
            Self::AccessControlCreation(msg) => {
                write!(f, "failed to create access control: {msg}")
            }
            Self::KeyGeneration(msg) => {
                write!(f, "failed to generate key: {msg}")
            }
            Self::KeyOperation(msg) => {
                write!(f, "key operation failed: {msg}")
            }
            Self::InvalidKeyData(msg) => {
                write!(f, "invalid key data: {msg}")
            }
            Self::AuthenticationFailed => {
                write!(f, "user authentication failed")
            }
            Self::AuthenticationCancelled => {
                write!(f, "authentication was cancelled by the user")
            }
            Self::AccessDenied => {
                write!(f, "access denied due to access control restrictions")
            }
            Self::SecurityFramework(code, msg) => {
                write!(f, "Security framework error ({code}): {msg}")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(target_os = "macos")]
impl Error {
    /// Create an error from a `Core Foundation` error reference.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `error` is a valid `CFErrorRef` or null.
    pub(crate) unsafe fn from_cf_error(error: core_foundation_sys::error::CFErrorRef) -> Self {
        use core_foundation::base::TCFType;
        use core_foundation::error::CFError;

        if error.is_null() {
            return Self::SecurityFramework(-1, "unknown error (null CFError)".to_string());
        }

        // SAFETY: We've verified the pointer is non-null, and the caller guarantees validity
        let cf_error = unsafe { CFError::wrap_under_create_rule(error) };
        let code = i32::try_from(cf_error.code()).unwrap_or(-1);
        let description = cf_error.description().to_string();

        // Map known error codes to specific error variants
        match code {
            -25293 => Self::AuthenticationFailed,  // errSecAuthFailed
            -128 => Self::AuthenticationCancelled, // userCanceledErr
            -25308 => Self::AccessDenied,          // errSecInteractionNotAllowed
            _ => Self::SecurityFramework(code, description),
        }
    }
}
