use p256::ecdh::{EphemeralSecret, SharedSecret};
use p256::PublicKey;
use thiserror::Error;
use rand::thread_rng;
use std::sync::Arc;


#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Secure Enclave not supported on this device")]
    SEUnsupported,
    
    #[error("Failed to create Secure Enclave key: {0}")]
    SEKeyCreationFailed(String),
    
    #[error("Failed to load Secure Enclave key: {0}")]
    SEKeyLoadFailed(String),
    
    #[error("Failed key agreement: {0}")]
    KeyAgreementFailed(String),
    
    #[cfg(target_os = "macos")]
    #[error("Security framework error: {0}")]
    SecurityFrameworkError(String),
}

pub trait Crypto: Send + Sync {
    fn is_secure_enclave_available(&self) -> bool;
    fn new_secure_enclave_private_key(&self, data_representation: Option<&[u8]>, access_control: Option<KeyAccessControl>) -> Result<Arc<dyn SecureEnclavePrivateKey>, CryptoError>;
    fn new_ephemeral_private_key(&self) -> EphemeralSecret;
}

pub trait SecureEnclavePrivateKey: Send + Sync {
    fn public_key(&self) -> PublicKey;
    fn data_representation(&self) -> Vec<u8>;
    fn shared_secret_from_key_agreement(&self, public_key_share: &PublicKey) -> Result<SharedSecret, CryptoError>;
}

#[derive(Debug, Clone, Copy)]
pub enum KeyAccessControl {
    None,
    Passcode,
    AnyBiometry,
    AnyBiometryOrPasscode,
    AnyBiometryAndPasscode,
    CurrentBiometry,
    CurrentBiometryAndPasscode,
}

// Default implementation for all platforms - will be extended later
pub struct DefaultCrypto {}

impl DefaultCrypto {
    pub fn new() -> Self {
        DefaultCrypto {}
    }
}

impl Crypto for DefaultCrypto {
    fn is_secure_enclave_available(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // This is a stub, actual implementation would check for Secure Enclave availability
            true
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
    
    fn new_secure_enclave_private_key(&self, data_representation: Option<&[u8]>, access_control: Option<KeyAccessControl>) -> Result<Arc<dyn SecureEnclavePrivateKey>, CryptoError> {
        #[cfg(target_os = "macos")]
        {
            // This is a stub, actual implementation would interact with the Secure Enclave
            Err(CryptoError::SEKeyCreationFailed("Not implemented yet".to_string()))
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Err(CryptoError::SEUnsupported)
        }
    }
    
    fn new_ephemeral_private_key(&self) -> EphemeralSecret {
        EphemeralSecret::random(&mut thread_rng())
    }
}

// Factory function to create the appropriate Crypto implementation
pub fn create_crypto() -> impl Crypto {
    DefaultCrypto::new()
} 