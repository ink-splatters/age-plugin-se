//! FFI bindings to Apple's `LocalAuthentication.framework`.
//!
//! This module provides bindings for `LAContext`, which is used to configure
//! biometric authentication prompts.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
// objc_msgSend is a variadic C function that we call with different signatures
#![allow(clashing_extern_declarations)]

use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::string::CFStringRef;
use std::ffi::c_void;

/// Opaque type representing an `LAContext` object.
pub type LAContextRef = *mut c_void;

/// `LAPolicy` enum values.
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum LAPolicy {
    /// Device owner authentication using biometry (Touch ID / Face ID).
    DeviceOwnerAuthenticationWithBiometrics = 1,
    /// Device owner authentication using biometry or passcode.
    DeviceOwnerAuthentication = 2,
    /// Device owner authentication using Watch.
    DeviceOwnerAuthenticationWithWatch = 3,
    /// Device owner authentication using biometry or Watch.
    DeviceOwnerAuthenticationWithBiometricsOrWatch = 4,
    /// Device owner authentication using Watch or companion devices.
    DeviceOwnerAuthenticationWithWristDetection = 5,
}

/// `LABiometryType` enum values.
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LABiometryType {
    /// No biometry is available.
    None = 0,
    /// Touch ID is available.
    TouchID = 1,
    /// Face ID is available.
    FaceID = 2,
    /// Optic ID is available (Apple Vision Pro).
    OpticID = 4,
}

// The LAContext class is an Objective-C class, so we need to use the ObjC runtime.
// For simplicity, we'll provide a minimal interface that works with the Security framework.

unsafe extern "C" {
    // Note: LAContext is an Objective-C class. We'll interact with it through
    // the Security framework's LAContext integration rather than direct ObjC calls.
    // The Security framework accepts LAContext objects via the
    // kSecUseAuthenticationContext attribute.

    /// Key for specifying an `LAContext` in Security framework calls.
    pub static kSecUseAuthenticationContext: CFStringRef;
}

/// Minimal `LAContext` wrapper that can be passed to Security framework.
///
/// For full `LAContext` functionality, you would need to use the `objc2` crate
/// or similar to interact with the Objective-C runtime directly. This wrapper
/// provides just enough functionality to work with the Security framework.
#[derive(Debug)]
pub struct LAContext {
    inner: LAContextRef,
}

impl LAContext {
    /// Create a new `LAContext`.
    ///
    /// Returns `None` if the context could not be created.
    pub fn new() -> Option<Self> {
        // Use the ObjC runtime to create an LAContext
        // This is a minimal implementation - for production, use objc2 crate
        unsafe {
            let class = objc_get_class(c"LAContext".as_ptr());
            if class.is_null() {
                return None;
            }

            let alloc_sel = sel_register_name(c"alloc".as_ptr());
            let init_sel = sel_register_name(c"init".as_ptr());

            let obj: LAContextRef = objc_msg_send_noargs(class, alloc_sel);
            if obj.is_null() {
                return None;
            }

            let obj: LAContextRef = objc_msg_send_noargs(obj, init_sel);
            if obj.is_null() {
                return None;
            }

            Some(Self { inner: obj })
        }
    }

    /// Get the raw pointer to the `LAContext`.
    ///
    /// This can be passed to the Security framework.
    #[must_use]
    pub const fn as_ptr(&self) -> LAContextRef {
        self.inner
    }

    /// Get the `LAContext` as a `CFTypeRef` for use with Core Foundation APIs.
    #[must_use]
    pub const fn as_cf_type_ref(&self) -> CFTypeRef {
        self.inner
    }

    /// Set the localized reason string shown to the user during authentication.
    pub fn set_localized_reason(&self, reason: &str) {
        unsafe {
            let sel = sel_register_name(c"setLocalizedReason:".as_ptr());
            let ns_string = create_ns_string(reason);
            if !ns_string.is_null() {
                objc_msg_send_one_arg(self.inner, sel, ns_string);
                objc_release(ns_string);
            }
        }
    }
}

impl Drop for LAContext {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                objc_release(self.inner);
            }
        }
    }
}

impl Default for LAContext {
    fn default() -> Self {
        // LAContext creation should succeed on macOS.
        // If it doesn't, we panic with a message about the ObjC runtime being unavailable.
        Self::new()
            .unwrap_or_else(|| panic!("Failed to create LAContext - ObjC runtime unavailable"))
    }
}

// Minimal ObjC runtime bindings (linked via Foundation framework in build.rs)
// We use separate functions for different arities to avoid issues with variadics
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn objc_get_class(name: *const i8) -> *mut c_void;
    fn sel_register_name(name: *const i8) -> *mut c_void;
    fn objc_release(obj: *mut c_void);
}

// ObjC message send with no arguments
unsafe fn objc_msg_send_noargs(obj: *mut c_void, sel: *mut c_void) -> *mut c_void {
    // We declare a specific version for no additional args
    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {
        fn objc_msgSend(obj: *mut c_void, sel: *mut c_void) -> *mut c_void;
    }
    unsafe { objc_msgSend(obj, sel) }
}

// ObjC message send with one argument
unsafe fn objc_msg_send_one_arg(obj: *mut c_void, sel: *mut c_void, arg: *mut c_void) {
    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {
        fn objc_msgSend(obj: *mut c_void, sel: *mut c_void, arg: *mut c_void);
    }
    unsafe { objc_msgSend(obj, sel, arg) }
}

// ObjC message send for NSString init
unsafe fn objc_msg_send_init_string(
    obj: *mut c_void,
    sel: *mut c_void,
    bytes: *const u8,
    len: usize,
    encoding: u64,
) -> *mut c_void {
    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {
        fn objc_msgSend(
            obj: *mut c_void,
            sel: *mut c_void,
            bytes: *const u8,
            len: usize,
            encoding: u64,
        ) -> *mut c_void;
    }
    unsafe { objc_msgSend(obj, sel, bytes, len, encoding) }
}

/// Create an `NSString` from a Rust string.
///
/// The caller is responsible for releasing the returned object.
unsafe fn create_ns_string(s: &str) -> *mut c_void {
    unsafe {
        let class = objc_get_class(c"NSString".as_ptr());
        if class.is_null() {
            return std::ptr::null_mut();
        }

        let alloc_sel = sel_register_name(c"alloc".as_ptr());
        let init_sel = sel_register_name(c"initWithBytes:length:encoding:".as_ptr());

        let obj: *mut c_void = objc_msg_send_noargs(class, alloc_sel);
        if obj.is_null() {
            return std::ptr::null_mut();
        }

        // NSUTF8StringEncoding = 4
        let obj: *mut c_void = objc_msg_send_init_string(obj, init_sel, s.as_ptr(), s.len(), 4);

        obj
    }
}
