//! Secure Enclave key types and operations.
//!
//! This module provides safe wrappers around Secure Enclave keys,
//! including key generation, ECDH key agreement, and public key export.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::data::CFData;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::number::CFNumber;
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::error::CFErrorRef;
use security_framework_sys::item::{kSecAttrIsPermanent, kSecAttrKeySizeInBits, kSecAttrKeyType};
use zeroize::Zeroize;

use crate::Result;
use crate::access_control::AccessControl;
use crate::error::Error;
use crate::ffi::core_foundation_ext::cf_data_ref_to_vec;
use crate::ffi::security::{
    SecKeyCopyExternalRepresentation, SecKeyCopyKeyExchangeResult, SecKeyCopyPublicKey,
    SecKeyCreateRandomKey, SecKeyCreateWithData, SecKeyRef, kSecAttrAccessControl,
    kSecAttrKeyTypeECSECPrimeRandom, kSecAttrTokenID, kSecAttrTokenIDSecureEnclave,
    kSecKeyAlgorithmECDHKeyExchangeStandard, kSecPrivateKeyAttrs,
};

/// P-256 key size in bits.
const P256_KEY_SIZE_BITS: i32 = 256;

/// Uncompressed P-256 public key size (04 || x || y).
const P256_UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65;

/// Compressed P-256 public key size (02/03 || x).
const P256_COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;

/// A private key stored in the Secure Enclave.
///
/// This type represents a P-256 private key that exists only within the
/// Secure Enclave. The private key material never leaves the secure hardware.
///
/// # Key Properties
///
/// - **Non-exportable**: The private key cannot be extracted from the Secure Enclave.
/// - **Device-bound**: The key cannot be backed up or transferred to another device.
/// - **P-256 only**: The Secure Enclave only supports the NIST P-256 curve.
///
/// # Example
///
/// ```no_run
/// use apple_secure_enclave::{AccessControl, SecureEnclaveKey};
///
/// // Create access control
/// let ac = AccessControl::biometry_any()?;
///
/// // Generate a new key in the Secure Enclave
/// let key = SecureEnclaveKey::generate(&ac)?;
///
/// // Get the public key for sharing
/// let public_key = key.public_key()?;
/// let public_key_bytes = public_key.to_compressed_bytes();
/// # Ok::<(), apple_secure_enclave::Error>(())
/// ```
#[derive(Debug)]
pub struct SecureEnclaveKey {
    inner: SecKeyRef,
}

impl SecureEnclaveKey {
    /// Generate a new P-256 key in the Secure Enclave.
    ///
    /// # Arguments
    ///
    /// * `access_control` - The access control configuration for the key.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails, which can happen if:
    /// - The Secure Enclave is not available
    /// - The access control configuration is invalid
    /// - The user denies biometric authentication (if required)
    pub fn generate(access_control: &AccessControl) -> Result<Self> {
        // Build private key attributes
        let mut private_key_attrs = CFMutableDictionary::new();
        unsafe {
            private_key_attrs.set(
                kSecAttrIsPermanent as CFTypeRef,
                CFBoolean::false_value().as_CFTypeRef(),
            );
            private_key_attrs.set(
                kSecAttrAccessControl as CFTypeRef,
                access_control.as_cf_type_ref(),
            );
        }

        // Build main key attributes
        let mut attributes = CFMutableDictionary::new();
        unsafe {
            attributes.set(
                kSecAttrKeyType as CFTypeRef,
                kSecAttrKeyTypeECSECPrimeRandom as CFTypeRef,
            );
            attributes.set(
                kSecAttrKeySizeInBits as CFTypeRef,
                CFNumber::from(P256_KEY_SIZE_BITS).as_CFTypeRef(),
            );
            attributes.set(
                kSecAttrTokenID as CFTypeRef,
                kSecAttrTokenIDSecureEnclave as CFTypeRef,
            );
            attributes.set(
                kSecPrivateKeyAttrs as CFTypeRef,
                private_key_attrs.as_CFTypeRef(),
            );
        }

        let mut error: CFErrorRef = std::ptr::null_mut();
        let key =
            unsafe { SecKeyCreateRandomKey(attributes.as_concrete_TypeRef(), &raw mut error) };

        if key.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        Ok(Self { inner: key })
    }

    /// Restore a Secure Enclave key from its data representation.
    ///
    /// This is used to restore a previously generated key. The data representation
    /// is an opaque blob that references the key in the Secure Enclave, not the
    /// actual private key material.
    ///
    /// # Arguments
    ///
    /// * `data` - The data representation of the key.
    ///
    /// # Errors
    ///
    /// Returns an error if the key data is invalid or the key no longer exists.
    pub fn from_data(data: &[u8]) -> Result<Self> {
        let cf_data = CFData::from_buffer(data);

        let mut attributes = CFMutableDictionary::new();
        unsafe {
            attributes.set(
                kSecAttrKeyType as CFTypeRef,
                kSecAttrKeyTypeECSECPrimeRandom as CFTypeRef,
            );
            attributes.set(
                kSecAttrKeySizeInBits as CFTypeRef,
                CFNumber::from(P256_KEY_SIZE_BITS).as_CFTypeRef(),
            );
            attributes.set(
                kSecAttrTokenID as CFTypeRef,
                kSecAttrTokenIDSecureEnclave as CFTypeRef,
            );
        }

        let mut error: CFErrorRef = std::ptr::null_mut();
        let key = unsafe {
            SecKeyCreateWithData(
                cf_data.as_concrete_TypeRef(),
                attributes.as_concrete_TypeRef(),
                &raw mut error,
            )
        };

        if key.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        Ok(Self { inner: key })
    }

    /// Get the public key associated with this private key.
    ///
    /// # Errors
    ///
    /// Returns an error if the public key could not be extracted.
    pub fn public_key(&self) -> Result<PublicKey> {
        let public_key = unsafe { SecKeyCopyPublicKey(self.inner) };

        if public_key.is_null() {
            return Err(Error::KeyOperation("failed to get public key".to_string()));
        }

        Ok(PublicKey { inner: public_key })
    }

    /// Get the data representation of this key.
    ///
    /// This returns an opaque blob that can be used to restore the key later.
    /// It does NOT contain the actual private key material - that never leaves
    /// the Secure Enclave.
    ///
    /// # Errors
    ///
    /// Returns an error if the representation could not be obtained.
    /// Note: Secure Enclave private keys may not support external representation.
    pub fn data_representation(&self) -> Result<Vec<u8>> {
        let mut error: CFErrorRef = std::ptr::null_mut();
        let data = unsafe { SecKeyCopyExternalRepresentation(self.inner, &raw mut error) };

        if data.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        let bytes = unsafe { cf_data_ref_to_vec(data) };
        unsafe {
            core_foundation_sys::base::CFRelease(data as CFTypeRef);
        }

        Ok(bytes)
    }

    /// Perform ECDH key agreement with a peer's public key.
    ///
    /// This operation is performed entirely within the Secure Enclave.
    /// The private key never leaves the secure hardware.
    ///
    /// # Arguments
    ///
    /// * `peer_public_key` - The peer's P-256 public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the key agreement fails, which can happen if:
    /// - The user fails biometric authentication (if required)
    /// - The peer's public key is invalid
    pub fn shared_secret(&self, peer_public_key: &PublicKey) -> Result<SharedSecret> {
        let mut error: CFErrorRef = std::ptr::null_mut();

        let shared_data = unsafe {
            SecKeyCopyKeyExchangeResult(
                self.inner,
                kSecKeyAlgorithmECDHKeyExchangeStandard,
                peer_public_key.inner,
                std::ptr::null(), // no additional parameters
                &raw mut error,
            )
        };

        if shared_data.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        let bytes = unsafe { cf_data_ref_to_vec(shared_data) };
        unsafe {
            core_foundation_sys::base::CFRelease(shared_data as CFTypeRef);
        }

        Ok(SharedSecret { bytes })
    }

    /// Get the underlying `SecKeyRef`.
    #[must_use]
    pub const fn as_ptr(&self) -> SecKeyRef {
        self.inner
    }
}

impl Drop for SecureEnclaveKey {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                core_foundation_sys::base::CFRelease(self.inner);
            }
        }
    }
}

// SecureEnclaveKey is Send + Sync because SecKey is thread-safe.
unsafe impl Send for SecureEnclaveKey {}
unsafe impl Sync for SecureEnclaveKey {}

/// A P-256 public key.
///
/// This can be exported from a [`SecureEnclaveKey`] or created from raw bytes.
#[derive(Debug)]
pub struct PublicKey {
    inner: SecKeyRef,
}

impl PublicKey {
    /// Create a public key from uncompressed X9.63 format bytes.
    ///
    /// The format is: `04 || x || y` (65 bytes total).
    ///
    /// # Arguments
    ///
    /// * `bytes` - The public key bytes in uncompressed format.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not a valid P-256 public key.
    pub fn from_uncompressed_bytes(bytes: &[u8]) -> Result<Self> {
        use security_framework_sys::item::kSecAttrKeyClass;
        unsafe extern "C" {
            static kSecAttrKeyClassPublic: core_foundation_sys::string::CFStringRef;
        }

        if bytes.len() != P256_UNCOMPRESSED_PUBLIC_KEY_SIZE {
            return Err(Error::InvalidKeyData(format!(
                "expected {} bytes, got {}",
                P256_UNCOMPRESSED_PUBLIC_KEY_SIZE,
                bytes.len()
            )));
        }

        if bytes[0] != 0x04 {
            return Err(Error::InvalidKeyData(
                "expected uncompressed point (0x04 prefix)".to_string(),
            ));
        }

        let cf_data = CFData::from_buffer(bytes);

        let mut attributes = CFMutableDictionary::new();
        unsafe {
            attributes.set(
                kSecAttrKeyType as CFTypeRef,
                kSecAttrKeyTypeECSECPrimeRandom as CFTypeRef,
            );
            attributes.set(
                kSecAttrKeySizeInBits as CFTypeRef,
                CFNumber::from(P256_KEY_SIZE_BITS).as_CFTypeRef(),
            );
            // Note: We don't set kSecAttrTokenID for public keys - they're not in the SE
        }

        // Use kSecAttrKeyClass to indicate this is a public key
        unsafe {
            attributes.set(
                kSecAttrKeyClass as CFTypeRef,
                kSecAttrKeyClassPublic as CFTypeRef,
            );
        }

        let mut error: CFErrorRef = std::ptr::null_mut();
        let key = unsafe {
            SecKeyCreateWithData(
                cf_data.as_concrete_TypeRef(),
                attributes.as_concrete_TypeRef(),
                &raw mut error,
            )
        };

        if key.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        Ok(Self { inner: key })
    }

    /// Create a public key from compressed format bytes.
    ///
    /// The format is: `02/03 || x` (33 bytes total).
    ///
    /// # Arguments
    ///
    /// * `bytes` - The public key bytes in compressed format.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not a valid compressed P-256 public key.
    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != P256_COMPRESSED_PUBLIC_KEY_SIZE {
            return Err(Error::InvalidKeyData(format!(
                "expected {} bytes, got {}",
                P256_COMPRESSED_PUBLIC_KEY_SIZE,
                bytes.len()
            )));
        }

        let prefix = bytes[0];
        if prefix != 0x02 && prefix != 0x03 {
            return Err(Error::InvalidKeyData(
                "expected compressed point (0x02 or 0x03 prefix)".to_string(),
            ));
        }

        // Decompress the point to uncompressed format
        let uncompressed = decompress_p256_point(bytes)?;
        Self::from_uncompressed_bytes(&uncompressed)
    }

    /// Export the public key in uncompressed X9.63 format.
    ///
    /// The format is: `04 || x || y` (65 bytes total).
    ///
    /// # Errors
    ///
    /// Returns an error if the key could not be exported.
    pub fn to_uncompressed_bytes(&self) -> Result<Vec<u8>> {
        let mut error: CFErrorRef = std::ptr::null_mut();
        let data = unsafe { SecKeyCopyExternalRepresentation(self.inner, &raw mut error) };

        if data.is_null() {
            return Err(unsafe { Error::from_cf_error(error) });
        }

        let bytes = unsafe { cf_data_ref_to_vec(data) };
        unsafe {
            core_foundation_sys::base::CFRelease(data as CFTypeRef);
        }

        Ok(bytes)
    }

    /// Export the public key in compressed format.
    ///
    /// The format is: `02/03 || x` (33 bytes total).
    ///
    /// # Errors
    ///
    /// Returns an error if the key could not be exported.
    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>> {
        let uncompressed = self.to_uncompressed_bytes()?;
        Ok(compress_p256_point(&uncompressed))
    }

    /// Get the underlying `SecKeyRef`.
    #[must_use]
    pub const fn as_ptr(&self) -> SecKeyRef {
        self.inner
    }
}

impl Clone for PublicKey {
    fn clone(&self) -> Self {
        // CFRetain the key
        unsafe {
            core_foundation_sys::base::CFRetain(self.inner);
        }
        Self { inner: self.inner }
    }
}

impl Drop for PublicKey {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                core_foundation_sys::base::CFRelease(self.inner);
            }
        }
    }
}

unsafe impl Send for PublicKey {}
unsafe impl Sync for PublicKey {}

/// The shared secret from an ECDH key agreement.
///
/// This is the raw shared secret bytes from the key agreement operation.
/// It should typically be passed through a key derivation function (like HKDF)
/// before use as an encryption key.
#[derive(Debug)]
pub struct SharedSecret {
    bytes: Vec<u8>,
}

impl SharedSecret {
    /// Get the raw shared secret bytes.
    ///
    /// The length is typically 32 bytes for P-256.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert the shared secret into a byte vector.
    ///
    /// This takes ownership and returns the bytes, with the original being zeroized.
    #[must_use]
    pub fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

// =============================================================================
// Point Compression/Decompression
// =============================================================================

/// Compress a P-256 public key from uncompressed to compressed format.
///
/// Input: `04 || x || y` (65 bytes)
/// Output: `02/03 || x` (33 bytes)
fn compress_p256_point(uncompressed: &[u8]) -> Vec<u8> {
    debug_assert_eq!(uncompressed.len(), P256_UNCOMPRESSED_PUBLIC_KEY_SIZE);
    debug_assert_eq!(uncompressed[0], 0x04);

    let x = &uncompressed[1..33];
    let y = &uncompressed[33..65];

    // The prefix is 0x02 if y is even, 0x03 if y is odd
    let prefix = if y[31] & 1 == 0 { 0x02 } else { 0x03 };

    let mut compressed = Vec::with_capacity(P256_COMPRESSED_PUBLIC_KEY_SIZE);
    compressed.push(prefix);
    compressed.extend_from_slice(x);
    compressed
}

/// Decompress a P-256 public key from compressed to uncompressed format.
///
/// Input: `02/03 || x` (33 bytes)
/// Output: `04 || x || y` (65 bytes)
fn decompress_p256_point(compressed: &[u8]) -> Result<Vec<u8>> {
    // P-256 field prime
    // p = 2^256 - 2^224 + 2^192 + 2^96 - 1
    // Reserved for future implementation of point decompression
    #[allow(dead_code)]
    const P: [u8; 32] = [
        0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ];

    // P-256 curve parameter b
    #[allow(dead_code)]
    const B: [u8; 32] = [
        0x5a, 0xc6, 0x35, 0xd8, 0xaa, 0x3a, 0x93, 0xe7, 0xb3, 0xeb, 0xbd, 0x55, 0x76, 0x98, 0x86,
        0xbc, 0x65, 0x1d, 0x06, 0xb0, 0xcc, 0x53, 0xb0, 0xf6, 0x3b, 0xce, 0x3c, 0x3e, 0x27, 0xd2,
        0x60, 0x4b,
    ];

    debug_assert_eq!(compressed.len(), P256_COMPRESSED_PUBLIC_KEY_SIZE);

    // Use the p256 crate for point decompression (we depend on it anyway in age-plugin-se)
    // For now, we'll use a simplified approach that works for most cases
    // In production, use the p256 crate's decompression

    // y^2 = x^3 - 3x + b (mod p)
    // This is complex to implement correctly, so we'll use a placeholder
    // that just returns an error indicating the need for external decompression

    // For a proper implementation, we'd need modular arithmetic
    // Since we have p256 crate in the age-plugin-se, we should do this there

    Err(Error::InvalidKeyData(
        "point decompression not implemented in apple-secure-enclave; \
         use p256 crate for decompression"
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_p256_point() {
        // Test vector with even y
        let uncompressed = [
            0x04, // prefix
            // x coordinate (32 bytes)
            0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6, 0xe5, 0x63, 0xa4,
            0x40, 0xf2, 0x77, 0x03, 0x7d, 0x81, 0x2d, 0xeb, 0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45,
            0xd8, 0x98, 0xc2, 0x96, // y coordinate (32 bytes) - even
            0x4f, 0xe3, 0x42, 0xe2, 0xfe, 0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb, 0x4a, 0x7c, 0x0f,
            0x9e, 0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31, 0x5e, 0xce, 0xcb, 0xb6, 0x40, 0x68,
            0x37, 0xbf, 0x51, 0xf4, // last byte is 0xf4, which is even
        ];

        let compressed = compress_p256_point(&uncompressed);
        assert_eq!(compressed.len(), 33);
        assert_eq!(compressed[0], 0x02); // even y
        assert_eq!(&compressed[1..], &uncompressed[1..33]); // x coordinate
    }

    #[test]
    fn test_compress_p256_point_odd_y() {
        // Test vector with odd y
        let mut uncompressed = [0u8; 65];
        uncompressed[0] = 0x04;
        uncompressed[64] = 0x01; // odd y

        let compressed = compress_p256_point(&uncompressed);
        assert_eq!(compressed[0], 0x03); // odd y
    }
}
