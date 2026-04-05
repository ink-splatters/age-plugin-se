//! Core Foundation helper utilities.
//!
//! This module provides helper functions for working with Core Foundation types.

use core_foundation::base::TCFType;
use core_foundation::data::CFData;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation_sys::base::CFTypeRef;

/// Extension trait for `CFMutableDictionary` to simplify setting key-value pairs.
pub trait CFMutableDictionaryExt {
    /// Set a key-value pair where both are CF types.
    ///
    /// # Safety
    ///
    /// The caller must ensure that both `key` and `value` are valid CF type references.
    unsafe fn set_cf(&mut self, key: CFTypeRef, value: CFTypeRef);

    /// Set a key-value pair where key is a CF type and value is a Rust value.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `key` is a valid CF type reference.
    unsafe fn set_cf_with<V: TCFType>(&mut self, key: CFTypeRef, value: &V);
}

impl CFMutableDictionaryExt for CFMutableDictionary {
    unsafe fn set_cf(&mut self, key: CFTypeRef, value: CFTypeRef) {
        self.set(key, value);
    }

    unsafe fn set_cf_with<V: TCFType>(&mut self, key: CFTypeRef, value: &V) {
        self.set(key, value.as_CFTypeRef());
    }
}

/// Convert a slice to a `CFData` object.
#[must_use]
pub fn slice_to_cf_data(slice: &[u8]) -> CFData {
    CFData::from_buffer(slice)
}

/// Convert a `CFData` object to a `Vec<u8>`.
#[must_use]
pub fn cf_data_to_vec(data: &CFData) -> Vec<u8> {
    data.bytes().to_vec()
}

/// Helper to safely extract bytes from a `CFData` reference.
///
/// # Safety
///
/// The caller must ensure that `data` is a valid `CFDataRef` and not null.
pub unsafe fn cf_data_ref_to_vec(data: core_foundation_sys::data::CFDataRef) -> Vec<u8> {
    if data.is_null() {
        return Vec::new();
    }

    // SAFETY: We've verified data is not null, and the caller guarantees it's a valid CFDataRef.
    // wrap_under_get_rule increments the reference count, so we own this reference.
    let cf_data = unsafe { CFData::wrap_under_get_rule(data) };
    cf_data.bytes().to_vec()
}
