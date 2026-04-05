//! Authentication context for Secure Enclave operations.
//!
//! This module provides a safe wrapper around `LAContext` for customizing
//! biometric authentication prompts.

use crate::ffi::local_auth::LAContext as RawLAContext;

/// Authentication context for customizing biometric prompts.
///
/// This wrapper allows you to configure the authentication prompt shown to
/// the user when they need to authenticate to use a Secure Enclave key.
///
/// # Example
///
/// ```no_run
/// use apple_secure_enclave::{AccessControl, AuthContext, SecureEnclaveKey};
///
/// let mut context = AuthContext::new();
/// context.set_localized_reason("Decrypt your message");
///
/// let ac = AccessControl::biometry_any()?;
/// let key = SecureEnclaveKey::generate_with_context(&ac, Some(context.into_inner()))?;
/// # Ok::<(), apple_secure_enclave::Error>(())
/// ```
#[derive(Debug)]
pub struct AuthContext {
    inner: Option<RawLAContext>,
}

impl AuthContext {
    /// Create a new authentication context.
    ///
    /// Returns `None` if the LAContext could not be created.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RawLAContext::new(),
        }
    }

    /// Set the localized reason string shown to the user during authentication.
    ///
    /// This string should explain why authentication is needed, such as
    /// "Decrypt your message" or "Access your private key".
    ///
    /// # Arguments
    ///
    /// * `reason` - The reason to display to the user.
    pub fn set_localized_reason(&mut self, reason: &str) {
        if let Some(ref ctx) = self.inner {
            ctx.set_localized_reason(reason);
        }
    }

    /// Check if the context was created successfully.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// Get the underlying LAContext.
    ///
    /// Returns `None` if the context was not created successfully.
    #[must_use]
    pub fn into_inner(self) -> Option<RawLAContext> {
        self.inner
    }

    /// Get a reference to the underlying LAContext.
    #[must_use]
    pub fn as_inner(&self) -> Option<&RawLAContext> {
        self.inner.as_ref()
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_creation() {
        let ctx = AuthContext::new();
        // On `macOS`, this should succeed. On other platforms, it will fail.
        // We don't assert on is_valid() because it depends on the platform.
        drop(ctx);
    }

    #[test]
    fn test_set_localized_reason() {
        let mut ctx = AuthContext::new();
        ctx.set_localized_reason("Test reason");
        // This should not panic
    }
}
