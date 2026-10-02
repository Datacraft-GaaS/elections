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

//! Post-quantum cryptography primitives.
//!
//! - `hybrid`: ML-DSA-65 (Dilithium3) + ECDSA P-384 hybrid signing (`pq-hybrid` feature).
//! - `kem`: ML-KEM-768 (Kyber768) key encapsulation (`pq-kem` feature).
//! - `slh`: SLH-DSA / SPHINCS+ archive signing (`pq-slh` feature).
//!
//! All sub-modules require the `liboqs` C library.  They are gated behind
//! feature flags so the base build stays pure Rust.
//!
//! Algorithm names follow the oqs 0.9 Rust API, which uses the pre-standardisation
//! NIST names (Dilithium, Kyber, Sphincs) mapping to the final FIPS names
//! (ML-DSA, ML-KEM, SLH-DSA) respectively.

/// ML-DSA-65 (Dilithium3) + ECDSA P-384 hybrid sign / verify.
///
/// A hybrid signature contains both a Dilithium3 and an ECDSA P-384 signature.
/// Verification requires both to be valid ("belt and suspenders" post-quantum
/// security model).
#[cfg(feature = "pq-hybrid")]
pub mod hybrid {
    use crate::CryptoError;
    use oqs::sig::{Algorithm as SigAlg, Sig};
    use p384::ecdsa::{
        signature::{Signer as EcdsaSigner, Verifier as EcdsaVerifier},
        Signature as EcdsaSignature, SigningKey as EcdsaSigningKey,
        VerifyingKey as EcdsaVerifyingKey,
    };
    use rand_core::OsRng;

    /// A hybrid key pair: Dilithium3 (ML-DSA-65) + ECDSA P-384.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use eel_crypto_rs::pq::hybrid::HybridKeyPair;
    /// let kp = HybridKeyPair::generate().unwrap();
    /// let sig = kp.sign(b"data").unwrap();
    /// assert!(kp.verify(b"data", &sig).is_ok());
    /// ```
    pub struct HybridKeyPair {
        dil_sk: oqs::sig::SecretKey,
        dil_pk: oqs::sig::PublicKey,
        ecdsa_sk: EcdsaSigningKey,
    }

    /// A hybrid signature (Dilithium3 component + ECDSA P-384 component).
    pub struct HybridSignature {
        /// Serialized Dilithium3 signature bytes.
        pub dilithium: Vec<u8>,
        /// ECDSA P-384 signature.
        pub ecdsa: EcdsaSignature,
    }

    /// A hybrid verifying key.
    pub struct HybridVerifyingKey {
        dil_pk: oqs::sig::PublicKey,
        ecdsa_pk: EcdsaVerifyingKey,
    }

    impl HybridKeyPair {
        /// Generate a new hybrid key pair.
        pub fn generate() -> Result<Self, CryptoError> {
            let sigalg =
                Sig::new(SigAlg::Dilithium3).map_err(|e| CryptoError::Internal(e.to_string()))?;
            let (dil_pk, dil_sk) = sigalg
                .keypair()
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            let ecdsa_sk = EcdsaSigningKey::random(&mut OsRng);
            Ok(HybridKeyPair {
                dil_sk,
                dil_pk,
                ecdsa_sk,
            })
        }

        /// Sign `message` with both algorithms.
        pub fn sign(&self, message: &[u8]) -> Result<HybridSignature, CryptoError> {
            let sigalg =
                Sig::new(SigAlg::Dilithium3).map_err(|e| CryptoError::Internal(e.to_string()))?;
            let dil_sig = sigalg
                .sign(message, &self.dil_sk)
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            let ecdsa_sig: EcdsaSignature = self.ecdsa_sk.sign(message);
            Ok(HybridSignature {
                dilithium: dil_sig.into_vec(),
                ecdsa: ecdsa_sig,
            })
        }

        /// Return the verifying (public) key for this pair.
        pub fn verifying_key(&self) -> HybridVerifyingKey {
            HybridVerifyingKey {
                dil_pk: self.dil_pk.clone(),
                ecdsa_pk: *self.ecdsa_sk.verifying_key(),
            }
        }

        /// Convenience: verify a hybrid signature.
        pub fn verify(&self, message: &[u8], sig: &HybridSignature) -> Result<(), CryptoError> {
            self.verifying_key().verify(message, sig)
        }
    }

    impl HybridVerifyingKey {
        /// Verify that `sig` is a valid hybrid signature over `message`.
        ///
        /// Both the Dilithium3 and ECDSA P-384 components must verify.
        pub fn verify(&self, message: &[u8], sig: &HybridSignature) -> Result<(), CryptoError> {
            let sigalg =
                Sig::new(SigAlg::Dilithium3).map_err(|e| CryptoError::Internal(e.to_string()))?;
            let dil_sig = sigalg
                .signature_from_bytes(&sig.dilithium)
                .ok_or(CryptoError::InvalidSignature)?;
            sigalg
                .verify(message, &dil_sig, &self.dil_pk)
                .map_err(|_| CryptoError::InvalidSignature)?;
            self.ecdsa_pk
                .verify(message, &sig.ecdsa)
                .map_err(|_| CryptoError::InvalidSignature)?;
            Ok(())
        }
    }
}

/// ML-KEM-768 (Kyber768) key encapsulation.
#[cfg(feature = "pq-kem")]
pub mod kem {
    use crate::CryptoError;
    use oqs::kem::{Algorithm as KemAlg, Kem};

    /// An ML-KEM-768 ciphertext.
    pub struct MlKemCiphertext {
        /// Serialized Kyber768 ciphertext bytes.
        pub ct_bytes: Vec<u8>,
    }

    /// Generate an ML-KEM-768 key pair.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let (pk, sk) = eel_crypto_rs::pq::kem::generate_keypair().unwrap();
    /// ```
    pub fn generate_keypair() -> Result<(oqs::kem::PublicKey, oqs::kem::SecretKey), CryptoError> {
        let kem = Kem::new(KemAlg::Kyber768).map_err(|e| CryptoError::Internal(e.to_string()))?;
        kem.keypair()
            .map_err(|e| CryptoError::Internal(e.to_string()))
    }

    /// Encapsulate to `pk`, returning (ciphertext, shared_secret).
    pub fn encapsulate(
        pk: &oqs::kem::PublicKey,
    ) -> Result<(MlKemCiphertext, Vec<u8>), CryptoError> {
        let kem = Kem::new(KemAlg::Kyber768).map_err(|e| CryptoError::Internal(e.to_string()))?;
        let (ct, ss) = kem
            .encapsulate(pk)
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        Ok((
            MlKemCiphertext {
                ct_bytes: ct.into_vec(),
            },
            ss.into_vec(),
        ))
    }

    /// Decapsulate to recover the shared secret.
    pub fn decapsulate(
        sk: &oqs::kem::SecretKey,
        ct: &MlKemCiphertext,
    ) -> Result<Vec<u8>, CryptoError> {
        let kem = Kem::new(KemAlg::Kyber768).map_err(|e| CryptoError::Internal(e.to_string()))?;
        let ct_obj = kem
            .ciphertext_from_bytes(&ct.ct_bytes)
            .ok_or_else(|| CryptoError::InvalidKey("bad ML-KEM ciphertext".to_owned()))?;
        kem.decapsulate(sk, &ct_obj)
            .map(|ss| ss.into_vec())
            .map_err(|e| CryptoError::Internal(e.to_string()))
    }
}

/// SLH-DSA / SPHINCS+ archive signing.
///
/// Uses SPHINCS+-SHAKE-128f-simple, the smallest SPHINCS+ variant in oqs 0.9.
/// Signatures are ~8 kB; suitable for long-term (15-year) archive retention.
#[cfg(feature = "pq-slh")]
pub mod slh {
    use crate::CryptoError;
    use oqs::sig::{Algorithm as SigAlg, Sig};

    /// Sign `message` with SPHINCS+-SHAKE-128f-simple.
    ///
    /// Returns `(public_key_bytes, signature_bytes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let (pk, sig) = eel_crypto_rs::pq::slh::sign(b"archive document").unwrap();
    /// ```
    pub fn sign(message: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let sigalg = Sig::new(SigAlg::SphincsShake128fSimple)
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        let (pk, sk) = sigalg
            .keypair()
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        let sig = sigalg
            .sign(message, &sk)
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        Ok((pk.into_vec(), sig.into_vec()))
    }

    /// Verify a SPHINCS+ signature.
    pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let sigalg = Sig::new(SigAlg::SphincsShake128fSimple)
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        let pk = sigalg
            .public_key_from_bytes(public_key)
            .ok_or_else(|| CryptoError::InvalidKey("bad SPHINCS+ public key".to_owned()))?;
        let sig = sigalg
            .signature_from_bytes(signature)
            .ok_or(CryptoError::InvalidSignature)?;
        sigalg
            .verify(message, &sig, &pk)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}
