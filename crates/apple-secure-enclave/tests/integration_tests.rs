#![allow(clippy::expect_used)]

//! Integration tests for apple-secure-enclave.
//!
//! These tests require `macOS` with Secure Enclave hardware (T2 or Apple Silicon).
//! Tests that require biometric authentication are marked with `#[ignore]`.

use apple_secure_enclave::is_available;

#[allow(unused_imports)]
use apple_secure_enclave::Error;

#[test]
fn test_is_available_returns_bool() {
    // This test just verifies the function doesn't panic
    let _ = is_available();
}

#[cfg(target_os = "macos")]
mod macos_tests {
    use apple_secure_enclave::{AccessControl, PublicKey, SecureEnclaveKey};

    /// Test that we can check SE availability.
    #[test]
    fn test_secure_enclave_availability_check() {
        let available = apple_secure_enclave::is_available();
        // Just verify it returns without panicking
        println!("Secure Enclave available: {available}");
    }

    /// Test `AccessControl` construction.
    #[test]
    fn test_access_control_construction() {
        // These tests verify `AccessControl` can be constructed.
        // They may fail on systems without Secure Enclave but shouldn't panic.

        match AccessControl::none() {
            Ok(ac) => println!("AccessControl::none() created: {ac:?}"),
            Err(e) => println!("AccessControl::none() failed (expected on non-SE): {e}"),
        }

        match AccessControl::passcode() {
            Ok(ac) => println!("AccessControl::passcode() created: {ac:?}"),
            Err(e) => println!("AccessControl::passcode() failed (expected on non-SE): {e}"),
        }

        match AccessControl::biometry_any() {
            Ok(ac) => println!("AccessControl::biometry_any() created: {ac:?}"),
            Err(e) => println!("AccessControl::biometry_any() failed (expected on non-SE): {e}"),
        }
    }

    /// Test key generation in Secure Enclave (requires SE hardware).
    #[test]
    #[ignore = "requires Secure Enclave hardware"]
    fn test_key_generation() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control = AccessControl::none().expect("Failed to create access control");
        let key = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key");

        // Verify we can get the public key
        let public_key = key.public_key().expect("Failed to get public key");

        // Verify the public key can be exported
        let uncompressed = public_key
            .to_uncompressed_bytes()
            .expect("Failed to export uncompressed");
        assert_eq!(uncompressed.len(), 65);
        assert_eq!(uncompressed[0], 0x04);

        let compressed = public_key
            .to_compressed_bytes()
            .expect("Failed to export compressed");
        assert_eq!(compressed.len(), 33);
        assert!(compressed[0] == 0x02 || compressed[0] == 0x03);
    }

    /// Test public key round-trip (export and import).
    #[test]
    #[ignore = "requires Secure Enclave hardware"]
    fn test_public_key_roundtrip() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control = AccessControl::none().expect("Failed to create access control");
        let key = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key");
        let public_key = key.public_key().expect("Failed to get public key");

        // Export uncompressed
        let uncompressed = public_key
            .to_uncompressed_bytes()
            .expect("Failed to export");

        // Re-import
        let imported = PublicKey::from_uncompressed_bytes(&uncompressed).expect("Failed to import");

        // Export again and compare
        let re_exported = imported
            .to_uncompressed_bytes()
            .expect("Failed to re-export");
        assert_eq!(uncompressed, re_exported);
    }

    /// Test ECDH key agreement (requires SE hardware).
    #[test]
    #[ignore = "requires Secure Enclave hardware"]
    fn test_ecdh_key_agreement() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control = AccessControl::none().expect("Failed to create access control");

        // Generate two keys
        let key1 = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key1");
        let key2 = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key2");

        // Get public keys
        let pub1 = key1.public_key().expect("Failed to get pub1");
        let pub2 = key2.public_key().expect("Failed to get pub2");

        // Perform key agreement
        let secret1 = key1
            .shared_secret(&pub2)
            .expect("Failed to compute shared secret 1");
        let secret2 = key2
            .shared_secret(&pub1)
            .expect("Failed to compute shared secret 2");

        // Shared secrets should match
        assert_eq!(secret1.as_bytes(), secret2.as_bytes());
        assert_eq!(secret1.as_bytes().len(), 32); // P-256 shared secret is 32 bytes
    }

    /// Test key generation with biometry (requires user interaction).
    #[test]
    #[ignore = "requires biometric authentication"]
    fn test_key_generation_with_biometry() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control =
            AccessControl::biometry_any().expect("Failed to create biometry access control");
        let _key = SecureEnclaveKey::generate(&access_control)
            .expect("Failed to generate key with biometry");
    }

    /// Test `SharedSecret` zeroization on drop.
    #[test]
    #[ignore = "requires Secure Enclave hardware"]
    fn test_shared_secret_zeroization() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control = AccessControl::none().expect("Failed to create access control");
        let key1 = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key1");
        let key2 = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key2");
        let pub2 = key2.public_key().expect("Failed to get pub2");

        let secret = key1
            .shared_secret(&pub2)
            .expect("Failed to compute shared secret");
        let bytes = secret.into_bytes();

        // Verify we got the bytes
        assert_eq!(bytes.len(), 32);
        // Note: We can't verify zeroization directly since the original is dropped,
        // but this test ensures into_bytes() works correctly.
    }

    /// Test public key from invalid bytes.
    #[test]
    fn test_public_key_from_invalid_bytes() {
        // Wrong length
        let result = PublicKey::from_uncompressed_bytes(&[0x04; 32]);
        assert!(result.is_err());

        // Wrong prefix
        let mut bytes = [0u8; 65];
        bytes[0] = 0x05; // Invalid prefix
        let result = PublicKey::from_uncompressed_bytes(&bytes);
        assert!(result.is_err());
    }

    /// Test compressed public key validation.
    #[test]
    fn test_compressed_public_key_validation() {
        // Wrong length
        let result = PublicKey::from_compressed_bytes(&[0x02; 32]);
        assert!(result.is_err());

        // Wrong prefix
        let mut bytes = [0u8; 33];
        bytes[0] = 0x04; // Wrong prefix for compressed
        let result = PublicKey::from_compressed_bytes(&bytes);
        assert!(result.is_err());

        // Note: from_compressed_bytes will fail because decompression is not implemented
        // in this crate (by design - use the p256 crate for that)
    }

    /// Test multiple key generations don't leak resources.
    #[test]
    #[ignore = "requires Secure Enclave hardware, takes time"]
    fn test_key_generation_no_leak() {
        if !apple_secure_enclave::is_available() {
            eprintln!("Skipping test: Secure Enclave not available");
            return;
        }

        let access_control = AccessControl::none().expect("Failed to create access control");

        // Generate many keys to check for resource leaks
        for i in 0..100 {
            let key = SecureEnclaveKey::generate(&access_control).expect("Failed to generate key");
            let _pub_key = key.public_key().expect("Failed to get public key");
            if i % 20 == 0 {
                println!("Generated {} keys", i + 1);
            }
        }
    }
}

/// Tests for non-`macOS` platforms (stub implementations).
#[cfg(not(target_os = "macos"))]
mod stub_tests {
    use apple_secure_enclave::{AccessControl, Error, SecureEnclaveKey};

    #[test]
    fn test_is_available_returns_false() {
        assert!(!apple_secure_enclave::is_available());
    }

    #[test]
    fn test_access_control_returns_not_available() {
        let result = AccessControl::biometry_any();
        assert!(matches!(result, Err(Error::NotAvailable)));

        let result = AccessControl::passcode();
        assert!(matches!(result, Err(Error::NotAvailable)));
    }
}
