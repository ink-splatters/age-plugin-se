//! Access control configuration for Secure Enclave keys.
//!
//! This module provides safe wrappers around `SecAccessControl` for configuring
//! access requirements for Secure Enclave keys.

use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::error::CFErrorRef;

use crate::Result;
use crate::error::Error;
use crate::ffi::security::{
    SecAccessControlCreateFlags, SecAccessControlCreateWithFlags, SecAccessControlRef,
    kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly, kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
};

/// Protection level for access control.
///
/// This determines when the key can be accessed based on device state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Protection {
    /// Key is accessible when the device is unlocked.
    /// This is the most common setting for interactive use.
    #[default]
    WhenUnlockedThisDeviceOnly,

    /// Key is accessible only when a passcode is set on the device.
    /// The key becomes inaccessible if the passcode is removed.
    WhenPasscodeSetThisDeviceOnly,
}

impl Protection {
    /// Get the Core Foundation constant for this protection level.
    fn as_cf_type_ref(self) -> CFTypeRef {
        unsafe {
            match self {
                Self::WhenUnlockedThisDeviceOnly => {
                    kSecAttrAccessibleWhenUnlockedThisDeviceOnly.cast()
                }
                Self::WhenPasscodeSetThisDeviceOnly => {
                    kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast()
                }
            }
        }
    }
}

/// Flags specifying access control requirements.
///
/// These flags determine what authentication is required to use the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessControlFlags(SecAccessControlCreateFlags);

impl AccessControlFlags {
    /// No additional authentication required beyond device unlock.
    pub const NONE: Self = Self(SecAccessControlCreateFlags::PRIVATE_KEY_USAGE);

    /// Require device passcode.
    pub const PASSCODE: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::DEVICE_PASSCODE),
    );

    /// Require any enrolled biometry (Touch ID / Face ID).
    /// Keys remain valid even if biometric data is updated.
    pub const BIOMETRY_ANY: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::BIOMETRY_ANY),
    );

    /// Require biometry OR passcode (user's choice).
    pub const BIOMETRY_OR_PASSCODE: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::USER_PRESENCE),
    );

    /// Require biometry AND passcode.
    pub const BIOMETRY_AND_PASSCODE: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::BIOMETRY_ANY)
            .union(SecAccessControlCreateFlags::DEVICE_PASSCODE),
    );

    /// Require current biometry enrollment.
    /// Keys become invalid if biometric data is updated.
    pub const BIOMETRY_CURRENT_SET: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::BIOMETRY_CURRENT_SET),
    );

    /// Require current biometry AND passcode.
    pub const BIOMETRY_CURRENT_SET_AND_PASSCODE: Self = Self(
        SecAccessControlCreateFlags::PRIVATE_KEY_USAGE
            .union(SecAccessControlCreateFlags::BIOMETRY_CURRENT_SET)
            .union(SecAccessControlCreateFlags::DEVICE_PASSCODE),
    );

    /// Get the underlying flags.
    #[must_use]
    pub const fn inner(self) -> SecAccessControlCreateFlags {
        self.0
    }
}

impl Default for AccessControlFlags {
    fn default() -> Self {
        Self::NONE
    }
}

/// Access control configuration for Secure Enclave keys.
///
/// This type wraps a `SecAccessControl` object and manages its lifetime.
/// It specifies the protection level and authentication requirements for keys.
///
/// # Example
///
/// ```no_run
/// use apple_secure_enclave::{AccessControl, AccessControlFlags, Protection};
///
/// // Create access control requiring Touch ID / Face ID
/// let ac = AccessControl::new(Protection::WhenUnlockedThisDeviceOnly, AccessControlFlags::BIOMETRY_ANY)?;
///
/// // Or use a convenience method
/// let ac = AccessControl::biometry_any()?;
/// # Ok::<(), apple_secure_enclave::Error>(())
/// ```
#[derive(Debug)]
pub struct AccessControl {
    inner: SecAccessControlRef,
}

impl AccessControl {
    /// Create a new access control with the specified protection level and flags.
    ///
    /// # Arguments
    ///
    /// * `protection` - The protection level determining when the key is accessible.
    /// * `flags` - The authentication requirements for using the key.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn new(protection: Protection, flags: AccessControlFlags) -> Result<Self> {
        let mut error: CFErrorRef = std::ptr::null_mut();

        let inner = unsafe {
            SecAccessControlCreateWithFlags(
                std::ptr::null(), // default allocator
                protection.as_cf_type_ref(),
                flags.inner().bits(),
                &raw mut error,
            )
        };

        if inner.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        Ok(Self { inner })
    }

    /// Create access control with no authentication (beyond device unlock).
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn none() -> Result<Self> {
        Self::new(
            Protection::WhenUnlockedThisDeviceOnly,
            AccessControlFlags::NONE,
        )
    }

    /// Create access control requiring device passcode.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn passcode() -> Result<Self> {
        Self::new(
            Protection::WhenPasscodeSetThisDeviceOnly,
            AccessControlFlags::PASSCODE,
        )
    }

    /// Create access control requiring any enrolled biometry (Touch ID / Face ID).
    ///
    /// Keys created with this access control remain valid even if the user
    /// adds or removes fingerprints or re-enrolls Face ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn biometry_any() -> Result<Self> {
        Self::new(
            Protection::WhenUnlockedThisDeviceOnly,
            AccessControlFlags::BIOMETRY_ANY,
        )
    }

    /// Create access control requiring biometry OR passcode.
    ///
    /// The user can authenticate with either biometry or passcode.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn biometry_or_passcode() -> Result<Self> {
        Self::new(
            Protection::WhenUnlockedThisDeviceOnly,
            AccessControlFlags::BIOMETRY_OR_PASSCODE,
        )
    }

    /// Create access control requiring biometry AND passcode.
    ///
    /// The user must authenticate with both biometry and passcode.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn biometry_and_passcode() -> Result<Self> {
        Self::new(
            Protection::WhenPasscodeSetThisDeviceOnly,
            AccessControlFlags::BIOMETRY_AND_PASSCODE,
        )
    }

    /// Create access control requiring current biometry enrollment.
    ///
    /// Keys created with this access control become invalid if the user
    /// changes their biometric data (adds/removes fingerprints, re-enrolls Face ID).
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn biometry_current_set() -> Result<Self> {
        Self::new(
            Protection::WhenUnlockedThisDeviceOnly,
            AccessControlFlags::BIOMETRY_CURRENT_SET,
        )
    }

    /// Create access control requiring current biometry AND passcode.
    ///
    /// # Errors
    ///
    /// Returns an error if the access control could not be created.
    pub fn biometry_current_set_and_passcode() -> Result<Self> {
        Self::new(
            Protection::WhenPasscodeSetThisDeviceOnly,
            AccessControlFlags::BIOMETRY_CURRENT_SET_AND_PASSCODE,
        )
    }

    /// Get the underlying `SecAccessControlRef`.
    #[must_use]
    pub const fn as_ptr(&self) -> SecAccessControlRef {
        self.inner
    }

    /// Get the access control as a `CFTypeRef`.
    #[must_use]
    pub const fn as_cf_type_ref(&self) -> CFTypeRef {
        self.inner
    }
}

impl Drop for AccessControl {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                core_foundation_sys::base::CFRelease(self.inner);
            }
        }
    }
}

// AccessControl is Send + Sync because SecAccessControl is an immutable CF type
// that can be safely shared across threads.
unsafe impl Send for AccessControl {}
unsafe impl Sync for AccessControl {}
