# apple-secure-enclave

Safe Rust bindings for Apple's Secure Enclave Processor (SEP).

## Overview

The Secure Enclave is a hardware security module embedded in Apple devices
(A7+ chips, T2, M1+) that provides:

- Hardware-isolated P256 key generation
- Key agreement (ECDH) without exposing private keys
- Biometric/passcode-gated access control
- Device-bound keys (non-exportable)

## Features

- **Safe abstractions** over the macOS Security and LocalAuthentication frameworks
- **Type-safe access control** configuration with all biometry/passcode options
- **Memory-safe key handling** with automatic zeroization of sensitive data
- **Cross-platform stubs** for non-macOS platforms (returns `Error::NotAvailable`)

## Platform Support

| Platform | Support |
| -------- | ------- |
| macOS 10.12+ with T2/Apple Silicon | Full support |
| macOS without Secure Enclave | `Error::NotAvailable` |
| Linux/Windows | Stub implementations |

## Usage

```rust
use apple_secure_enclave::{AccessControl, SecureEnclaveKey};

// Check availability
if !apple_secure_enclave::is_available() {
    eprintln!("Secure Enclave not available");
    return;
}

// Create access control requiring biometry
let ac = AccessControl::biometry_any()?;

// Generate a key in the Secure Enclave
let key = SecureEnclaveKey::generate(&ac)?;

// Get the public key for sharing
let public_key = key.public_key()?;
let public_key_bytes = public_key.to_compressed_bytes()?;

// Perform ECDH with a peer's public key
let peer_public_key = PublicKey::from_compressed_bytes(&peer_bytes)?;
let shared_secret = key.shared_secret(&peer_public_key)?;
```

## Access Control Options

| Method | Description |
| ------ | ----------- |
| `AccessControl::none()` | No authentication beyond device unlock |
| `AccessControl::passcode()` | Require device passcode |
| `AccessControl::biometry_any()` | Require Touch ID / Face ID |
| `AccessControl::biometry_or_passcode()` | Biometry OR passcode |
| `AccessControl::biometry_and_passcode()` | Biometry AND passcode |
| `AccessControl::biometry_current_set()` | Current biometry (key invalidated on biometry change) |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
