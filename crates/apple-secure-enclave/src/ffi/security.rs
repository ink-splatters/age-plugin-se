//! FFI bindings to Apple's `Security.framework`.
//!
//! This module provides the low-level bindings needed for Secure Enclave operations
//! that are not exposed by the `security-framework-sys` crate.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use core_foundation_sys::base::{CFAllocatorRef, CFTypeRef, OSStatus};
use core_foundation_sys::data::CFDataRef;
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::error::CFErrorRef;
use core_foundation_sys::string::CFStringRef;

/// Opaque type representing a cryptographic key.
pub type SecKeyRef = CFTypeRef;

/// Opaque type representing an access control configuration.
pub type SecAccessControlRef = CFTypeRef;

// =============================================================================
// Key Type Constants
// =============================================================================

unsafe extern "C" {
    /// Key type for elliptic curve keys using the secp256r1 curve (P-256).
    pub static kSecAttrKeyTypeECSECPrimeRandom: CFStringRef;

    /// Token ID for keys stored in the Secure Enclave.
    pub static kSecAttrTokenIDSecureEnclave: CFStringRef;

    /// Attribute key for specifying the token ID.
    pub static kSecAttrTokenID: CFStringRef;

    /// Attribute key for specifying private key attributes.
    pub static kSecPrivateKeyAttrs: CFStringRef;

    /// Attribute key for specifying the access control.
    pub static kSecAttrAccessControl: CFStringRef;

    /// Protection level: accessible when unlocked, this device only.
    pub static kSecAttrAccessibleWhenUnlockedThisDeviceOnly: CFStringRef;

    /// Protection level: accessible after first unlock, this device only.
    pub static kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly: CFStringRef;

    /// Protection level: accessible when passcode set, this device only.
    pub static kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly: CFStringRef;
}

// =============================================================================
// Key Algorithm Constants
// =============================================================================

unsafe extern "C" {
    /// Algorithm for ECDH key exchange with standard X9.63 format.
    pub static kSecKeyAlgorithmECDHKeyExchangeStandard: CFStringRef;

    /// Algorithm for ECDH key exchange with cofactor.
    pub static kSecKeyAlgorithmECDHKeyExchangeCofactor: CFStringRef;
}

// =============================================================================
// Access Control Create Flags
// =============================================================================

bitflags::bitflags! {
    /// Flags for creating access control configurations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SecAccessControlCreateFlags: u64 {
        /// Constraint to access an item with Touch ID for any enrolled fingers,
        /// or Face ID.
        const BIOMETRY_ANY = 1 << 1;

        /// Constraint to access an item with Touch ID for currently enrolled fingers,
        /// or Face ID with the current enrollment. Updating biometric data invalidates the key.
        const BIOMETRY_CURRENT_SET = 1 << 3;

        /// Constraint to access an item with the device passcode.
        const DEVICE_PASSCODE = 1 << 4;

        /// Constraint to access an item with either biometry or passcode.
        const USER_PRESENCE = 1 << 0;

        /// Enable using a private key for operations.
        const PRIVATE_KEY_USAGE = 1 << 30;

        /// Create access control for an application-provided password.
        const APPLICATION_PASSWORD = 1 << 31;
    }
}

// =============================================================================
// SecAccessControl Functions
// =============================================================================

unsafe extern "C" {
    /// Creates an access control object with the specified protection and flags.
    ///
    /// # Parameters
    ///
    /// * `allocator` - The allocator to use, or `NULL` for the default allocator.
    /// * `protection` - The protection level (e.g., `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`).
    /// * `flags` - Access control flags specifying biometry, passcode requirements, etc.
    /// * `error` - On return, contains an error if the operation failed.
    ///
    /// # Returns
    ///
    /// A new access control object, or `NULL` if creation failed.
    pub fn SecAccessControlCreateWithFlags(
        allocator: CFAllocatorRef,
        protection: CFTypeRef,
        flags: u64, // SecAccessControlCreateFlags::bits()
        error: *mut CFErrorRef,
    ) -> SecAccessControlRef;
}

// =============================================================================
// SecKey Functions
// =============================================================================

unsafe extern "C" {
    /// Creates a new random cryptographic key.
    ///
    /// # Parameters
    ///
    /// * `parameters` - A dictionary of key generation parameters.
    /// * `error` - On return, contains an error if the operation failed.
    ///
    /// # Returns
    ///
    /// A new key reference, or `NULL` if creation failed.
    pub fn SecKeyCreateRandomKey(parameters: CFDictionaryRef, error: *mut CFErrorRef) -> SecKeyRef;

    /// Creates a key from external representation data.
    ///
    /// # Parameters
    ///
    /// * `keyData` - The external representation of the key.
    /// * `attributes` - A dictionary of key attributes.
    /// * `error` - On return, contains an error if the operation failed.
    ///
    /// # Returns
    ///
    /// A new key reference, or `NULL` if creation failed.
    pub fn SecKeyCreateWithData(
        keyData: CFDataRef,
        attributes: CFDictionaryRef,
        error: *mut CFErrorRef,
    ) -> SecKeyRef;

    /// Returns the public key from a private key.
    ///
    /// # Parameters
    ///
    /// * `key` - The private key.
    ///
    /// # Returns
    ///
    /// The public key, or `NULL` if the operation failed.
    pub fn SecKeyCopyPublicKey(key: SecKeyRef) -> SecKeyRef;

    /// Returns the external representation of a key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to export.
    /// * `error` - On return, contains an error if the operation failed.
    ///
    /// # Returns
    ///
    /// The key data in X9.63 format, or `NULL` if the operation failed.
    pub fn SecKeyCopyExternalRepresentation(key: SecKeyRef, error: *mut CFErrorRef) -> CFDataRef;

    /// Performs a key exchange operation.
    ///
    /// # Parameters
    ///
    /// * `privateKey` - The private key to use.
    /// * `algorithm` - The algorithm to use (e.g., `kSecKeyAlgorithmECDHKeyExchangeStandard`).
    /// * `publicKey` - The peer's public key.
    /// * `parameters` - Additional parameters (usually `NULL`).
    /// * `error` - On return, contains an error if the operation failed.
    ///
    /// # Returns
    ///
    /// The shared secret data, or `NULL` if the operation failed.
    pub fn SecKeyCopyKeyExchangeResult(
        privateKey: SecKeyRef,
        algorithm: CFStringRef,
        publicKey: SecKeyRef,
        parameters: CFDictionaryRef,
        error: *mut CFErrorRef,
    ) -> CFDataRef;

    /// Checks if a key supports a specific algorithm.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to check.
    /// * `operation` - The operation type.
    /// * `algorithm` - The algorithm to check.
    ///
    /// # Returns
    ///
    /// `true` if the key supports the algorithm, `false` otherwise.
    pub fn SecKeyIsAlgorithmSupported(
        key: SecKeyRef,
        operation: SecKeyOperationType,
        algorithm: CFStringRef,
    ) -> bool;
}

/// Key operation types for `SecKeyIsAlgorithmSupported`.
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecKeyOperationType {
    /// Signing operation.
    Sign = 0,
    /// Verification operation.
    Verify = 1,
    /// Encryption operation.
    Encrypt = 2,
    /// Decryption operation.
    Decrypt = 3,
    /// Key exchange operation.
    KeyExchange = 4,
}

// =============================================================================
// OSStatus Error Codes
// =============================================================================

/// Authentication failed (e.g., wrong passcode, biometry not recognized).
pub const ERR_SEC_AUTH_FAILED: OSStatus = -25293;

/// User cancelled the operation.
pub const ERR_SEC_USER_CANCELED: OSStatus = -128;

/// Interaction with the user is not allowed.
pub const ERR_SEC_INTERACTION_NOT_ALLOWED: OSStatus = -25308;

/// The Secure Enclave is not available.
pub const ERR_SEC_MISSING_ENTITLEMENT: OSStatus = -34018;
