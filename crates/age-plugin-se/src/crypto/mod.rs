//! Cryptographic operations for age-plugin-se.
//!
//! This module provides:
//! - P256 ECDH key agreement
//! - ChaCha20-Poly1305 AEAD for file key wrapping
//! - HKDF-SHA256 for key derivation
//! - HMAC-SHA256/SHA256 for recipient tags

pub mod aead;
pub mod kdf;
pub mod p256;

pub use aead::{unwrap_file_key, wrap_file_key};
pub use kdf::{derive_wrap_key, recipient_tag};
pub use p256::{compress_public_key, decompress_public_key, generate_ephemeral_keypair, p256_ecdh};
