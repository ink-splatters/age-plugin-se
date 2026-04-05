//! FFI bindings to Apple's `Security` and `LocalAuthentication` frameworks.
//!
//! This module provides low-level bindings to the `macOS` `Security.framework`
//! and `LocalAuthentication.framework`. These bindings are used internally
//! by the safe wrapper types.

// FFI modules may export items that aren't all used by this crate but may be
// useful for consumers or future development.
#[allow(dead_code)]
pub mod core_foundation_ext;
#[allow(dead_code)]
pub mod local_auth;
#[allow(dead_code)]
pub mod security;

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::number::CFNumber;
use core_foundation_sys::base::CFTypeRef;
use security_framework_sys::item::{kSecAttrKeySizeInBits, kSecAttrKeyType};

use self::security::{
    SecKeyCreateRandomKey, kSecAttrKeyTypeECSECPrimeRandom, kSecAttrTokenID,
    kSecAttrTokenIDSecureEnclave,
};

/// Check if the Secure Enclave is available on this device.
///
/// This attempts to create a test key in the Secure Enclave to verify availability.
pub fn is_secure_enclave_available() -> bool {
    // Build attributes for a test key
    let mut attributes = CFMutableDictionary::new();

    unsafe {
        attributes.set(
            kSecAttrKeyType as CFTypeRef,
            kSecAttrKeyTypeECSECPrimeRandom as CFTypeRef,
        );
        attributes.set(
            kSecAttrKeySizeInBits as CFTypeRef,
            CFNumber::from(256_i32).as_CFTypeRef(),
        );
        attributes.set(
            kSecAttrTokenID as CFTypeRef,
            kSecAttrTokenIDSecureEnclave as CFTypeRef,
        );
        // Don't persist the key
        attributes.set(
            security_framework_sys::item::kSecAttrIsPermanent as CFTypeRef,
            CFBoolean::false_value().as_CFTypeRef(),
        );
    }

    // Try to create a key - if it fails with certain errors, SE is not available
    let mut error: core_foundation_sys::error::CFErrorRef = std::ptr::null_mut();
    let key = unsafe { SecKeyCreateRandomKey(attributes.as_concrete_TypeRef(), &raw mut error) };

    if key.is_null() {
        // SE not available
        if !error.is_null() {
            unsafe {
                core_foundation_sys::base::CFRelease(error as CFTypeRef);
            }
        }
        return false;
    }

    // Clean up the test key
    unsafe {
        core_foundation_sys::base::CFRelease(key as CFTypeRef);
    }

    true
}
