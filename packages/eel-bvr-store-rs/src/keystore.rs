// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The signing-key seam.
//!
//! The instance key is what makes a record's provenance *provable* rather than
//! merely asserted. Where that key lives is a hardware decision, so the store
//! depends on this trait rather than on a concrete keystore:
//!
//! - In production the implementation should be a TPM 2.0 or PKCS#11 backed
//!   store, so the private key is non-exportable and a stolen disk yields
//!   nothing. ADR-0026 and ADR-0005 both point at PKCS#11, and
//!   `eel-crypto-rs::pkcs11_pool` is the existing entry point.
//! - [`SoftwareKeyStore`] keeps the key in process memory. It exists so tests
//!   and local development need no hardware, and it is **not** acceptable in a
//!   deployed station: anyone who can read the process can extract the key.

use eel_crypto_rs::ed25519;

/// Signs and verifies with the instance key.
pub trait KeyStore {
    /// Signs `message`, returning a 64-byte Ed25519 signature.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, eel_crypto_rs::CryptoError>;

    /// Returns the 32-byte public key.
    fn public_key(&self) -> [u8; 32];
}

/// In-memory key store. For tests and local development only.
pub struct SoftwareKeyStore {
    key: ed25519::Ed25519SigningKey,
}

impl SoftwareKeyStore {
    /// Generates a fresh key.
    pub fn generate() -> Self {
        Self {
            key: ed25519::generate_keypair(),
        }
    }
}

impl KeyStore for SoftwareKeyStore {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, eel_crypto_rs::CryptoError> {
        Ok(ed25519::sign(&self.key, message).to_bytes().to_vec())
    }

    fn public_key(&self) -> [u8; 32] {
        ed25519::public_key(&self.key)
    }
}
