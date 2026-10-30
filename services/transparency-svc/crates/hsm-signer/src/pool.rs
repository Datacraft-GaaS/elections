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

//! Role-scoped PKCS#11 pool wrapper.
//!
//! Selects vendor at startup via `EEL_HSM_VENDOR` env:
//! - `softhsm` (default / dev): SoftHSMv2 shared library
//! - `luna7`: Thales Luna 7 network HSM
//! - `utimaco`: Utimaco network HSM
//!
//! Signing uses CKM_ECDSA (P-384) for all roles.

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};

use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, AttributeType, ObjectClass};
use eel_crypto_rs::pkcs11_pool::{find_key_by_role, PkcsPool, PkcsPoolConfig};
use eel_crypto_rs::{CryptoError, KeyRole};

/// Default SoftHSM paths per platform.
#[cfg(target_os = "linux")]
const SOFTHSM_DEFAULT: &str = "/usr/lib/softhsm/libsofthsm2.so";
#[cfg(target_os = "macos")]
const SOFTHSM_DEFAULT: &str = "/usr/local/lib/softhsm/libsofthsm2.so";

/// HSM vendor selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HsmVendor {
    SoftHsm,
    Luna7,
    Utimaco,
}

impl HsmVendor {
    pub fn from_env() -> Self {
        match std::env::var("EEL_HSM_VENDOR")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "luna7" => Self::Luna7,
            "utimaco" => Self::Utimaco,
            _ => Self::SoftHsm,
        }
    }

    pub fn module_path(&self) -> PathBuf {
        match self {
            Self::SoftHsm => std::env::var("PKCS11_MODULE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(SOFTHSM_DEFAULT)),
            Self::Luna7 => std::env::var("PKCS11_MODULE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/usr/lib/libCryptoki2_64.so")),
            Self::Utimaco => std::env::var("PKCS11_MODULE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/opt/utimaco/lib/libcs_pkcs11_R3.so")),
        }
    }
}

/// Per-service pool wrapper shared across all role operations.
///
/// cryptoki 0.11's `Session` is deliberately `!Send`. The gRPC handlers call
/// `spawn_blocking`, which requires the pool to be `Send`, so the pool lives on
/// its own thread and this type carries a channel to it.
#[derive(Clone)]
pub struct RolePool {
    tx: Sender<HsmJob>,
    pub vendor: HsmVendor,
}

enum HsmJob {
    Sign {
        role: KeyRole,
        payload: Vec<u8>,
        reply: Sender<Result<(Vec<u8>, String, String), CryptoError>>,
    },
    Verify {
        role: KeyRole,
        payload: Vec<u8>,
        signature: Vec<u8>,
        reply: Sender<Result<bool, CryptoError>>,
    },
}

impl RolePool {
    /// Open the pool.
    ///
    /// `slot_id` is the PKCS#11 slot index (0 for SoftHSM default slot).
    /// `pin` is the user PIN.
    pub fn open(slot_id: u64, pin: String) -> Result<Self, CryptoError> {
        let vendor = HsmVendor::from_env();
        let config = PkcsPoolConfig {
            module_path: vendor.module_path(),
            slot_id,
            pin,
            max_sessions: 8,
        };
        let tx = spawn_worker(config)?;
        Ok(Self { tx, vendor })
    }

    /// Sign `payload` with the key for `role`.
    ///
    /// Returns `(signature_bytes, key_id_hex, algorithm)`.
    pub fn sign(
        &self,
        role: KeyRole,
        payload: &[u8],
    ) -> Result<(Vec<u8>, String, String), CryptoError> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(HsmJob::Sign {
                role,
                payload: payload.to_vec(),
                reply,
            })
            .map_err(|_| CryptoError::Internal("HSM worker stopped".into()))?;
        rx.recv()
            .map_err(|_| CryptoError::Internal("HSM worker dropped reply".into()))?
    }

    /// Verify a signature for `role`.
    ///
    /// Returns `Ok(true)` for valid, `Ok(false)` for invalid, `Err` for internal errors.
    pub fn verify(
        &self,
        role: KeyRole,
        payload: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(HsmJob::Verify {
                role,
                payload: payload.to_vec(),
                signature: signature.to_vec(),
                reply,
            })
            .map_err(|_| CryptoError::Internal("HSM worker stopped".into()))?;
        rx.recv()
            .map_err(|_| CryptoError::Internal("HSM worker dropped reply".into()))?
    }
}

fn spawn_worker(config: PkcsPoolConfig) -> Result<Sender<HsmJob>, CryptoError> {
    let (tx, rx) = mpsc::channel::<HsmJob>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), CryptoError>>();

    std::thread::Builder::new()
        .name("eel-hsm-signer".to_string())
        .spawn(move || {
            let pool = match PkcsPool::open(config) {
                Ok(pool) => pool,
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                    return;
                }
            };
            let _ = ready_tx.send(Ok(()));

            while let Ok(job) = rx.recv() {
                match job {
                    HsmJob::Sign {
                        role,
                        payload,
                        reply,
                    } => {
                        let _ = reply.send(sign_with_pool(&pool, role, &payload));
                    }
                    HsmJob::Verify {
                        role,
                        payload,
                        signature,
                        reply,
                    } => {
                        let _ = reply.send(verify_with_pool(&pool, role, &payload, &signature));
                    }
                }
            }
        })
        .map_err(|e| CryptoError::Internal(format!("spawn HSM worker: {e}")))?;

    ready_rx
        .recv()
        .map_err(|_| CryptoError::Internal("HSM worker failed to start".into()))??;
    Ok(tx)
}

fn sign_with_pool(
    pool: &PkcsPool,
    role: KeyRole,
    payload: &[u8],
) -> Result<(Vec<u8>, String, String), CryptoError> {
    let session = pool.get()?;
    let handle = find_key_by_role(session.session(), role)?;

    let signature = session
        .session()
        .sign(&Mechanism::Ecdsa, handle, payload)
        .map_err(|e| CryptoError::Internal(format!("sign failed: {e}")))?;

    let key_id = read_key_id(session.session(), handle)?;

    Ok((signature, key_id, "ECDSA-P384".to_string()))
}

fn verify_with_pool(
    pool: &PkcsPool,
    role: KeyRole,
    payload: &[u8],
    signature: &[u8],
) -> Result<bool, CryptoError> {
    let session = pool.get()?;
    // Verification uses the public key; find by label + PUBLIC_KEY class.
    let label = role.label();
    let template = vec![
        Attribute::Label(label.as_bytes().to_vec()),
        Attribute::Class(ObjectClass::PUBLIC_KEY),
    ];
    let handles = session
        .session()
        .find_objects(&template)
        .map_err(|e| CryptoError::Internal(format!("find public key: {e}")))?;
    let handle = handles.into_iter().next().ok_or_else(|| {
        CryptoError::InvalidKey(format!("no public key for role {:?}", label))
    })?;

    match session
        .session()
        .verify(&Mechanism::Ecdsa, handle, payload, signature)
    {
        Ok(()) => Ok(true),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("SignatureInvalid") || msg.contains("signature") {
                Ok(false)
            } else {
                Err(CryptoError::Internal(format!("verify error: {msg}")))
            }
        }
    }
}

fn read_key_id(
    session: &cryptoki::session::Session,
    handle: cryptoki::object::ObjectHandle,
) -> Result<String, CryptoError> {
    let attrs = session
        .get_attributes(handle, &[AttributeType::Id])
        .map_err(|e| CryptoError::Internal(format!("get_attributes: {e}")))?;

    for attr in attrs {
        if let Attribute::Id(id) = attr {
            return Ok(hex::encode(id));
        }
    }
    // Fallback: debug representation of the handle.
    Ok(format!("{handle:?}"))
}
