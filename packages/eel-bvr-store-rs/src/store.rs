//! The offline store: append, verify, encrypt, decrypt.

use eel_crypto_rs::aead;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::identity::InstanceIdentity;
use crate::keystore::{KeyStore, SoftwareKeyStore};
use crate::provenance::Provenance;
use crate::record::{Record, RecordId, TamperError, verify_chain};
use crate::sync::{Batch, SyncState};

/// Why a store operation failed.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The store's integrity check failed.
    #[error("store integrity check failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Tampered(Vec<TamperError>),
    /// A crypto primitive failed.
    #[error("crypto failure: {0}")]
    Crypto(#[from] eel_crypto_rs::CryptoError),
    /// The stored ciphertext was malformed or truncated.
    #[error("record {0} is not a well-formed sealed blob")]
    Malformed(String),
}

/// An offline registration store belonging to one station instance.
pub struct Store {
    identity: InstanceIdentity,
    keys: SoftwareKeyStore,
    records: Vec<Record>,
    data_key: [u8; 32],
}

impl Store {
    /// Creates a new store for a freshly minted instance.
    pub fn new(station_label: &str) -> Result<Self, StoreError> {
        let keys = SoftwareKeyStore::generate();
        let identity = InstanceIdentity {
            instance_id: Uuid::now_v7(),
            station_label: station_label.to_owned(),
            public_key_b64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                keys.public_key(),
            ),
        };
        Ok(Self {
            identity,
            keys,
            records: Vec::new(),
            data_key: aead::generate_data_key()?,
        })
    }

    /// The station's unique identity.
    pub fn identity(&self) -> &InstanceIdentity {
        &self.identity
    }

    /// The 32-byte Ed25519 public key records are signed with.
    ///
    /// Published in [`InstanceIdentity::public_key_b64`] so a server or auditor
    /// can verify this store's signatures without holding the private key.
    pub fn public_key(&self) -> [u8; 32] {
        self.keys.public_key()
    }

    /// Number of records held.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the store holds no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Appends a value with its provenance, chaining and signing it.
    ///
    /// The value is encrypted before it is stored, so an on-disk store never
    /// holds citizen PII in the clear.
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        data_item: &str,
        data_class: &str,
        value: &serde_json::Value,
        provenance: &Provenance,
    ) -> Result<RecordId, StoreError> {
        // Mint the id first: the record id is bound into the associated data,
        // so sealing has to happen after the id exists, or decryption could
        // never reproduce the same associated data.
        let record_id = Uuid::now_v7();
        let sealed = aead::seal(
            &self.data_key,
            &serde_json::to_vec(value).unwrap_or_default(),
            self.aad(data_item, data_class, Some(record_id)).as_bytes(),
        )?;

        let prev_hash = self.records.last().map(|r| r.this_hash.clone()).unwrap_or_default();
        let record = Record {
            record_id,
            data_item: data_item.to_owned(),
            data_class: data_class.to_owned(),
            value: serde_json::Value::String(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                sealed,
            )),
            provenance: provenance.clone(),
            prev_hash,
            this_hash: String::new(),
            signature: String::new(),
            synced: false,
        }
        .seal(&self.keys)?;

        let id = record.record_id;
        self.records.push(record);
        Ok(id)
    }

    /// Verifies content hashes, chain linkage, and signatures across the store.
    pub fn verify(&self) -> Result<(), StoreError> {
        verify_chain(&self.records, &self.keys.public_key()).map_err(StoreError::Tampered)
    }

    /// Decrypts a record's value.
    pub fn value_of(&self, id: &RecordId) -> Result<serde_json::Value, StoreError> {
        let record = self
            .records
            .iter()
            .find(|r| &r.record_id == id)
            .ok_or_else(|| StoreError::Malformed(id.to_string()))?;
        let sealed = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            record.value.as_str().unwrap_or_default(),
        )
        .map_err(|_| StoreError::Malformed(id.to_string()))?;
        let aad = self.aad(&record.data_item, &record.data_class, Some(*id));
        let plain = aead::open(&self.data_key, &sealed, aad.as_bytes())
            .map_err(|_| StoreError::Tampered(vec![TamperError::Undecryptable {
                id: id.to_string(),
            }]))?;
        Ok(serde_json::from_slice(&plain).unwrap_or(serde_json::Value::Null))
    }

    /// All records, in capture order.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Mutable access to the records.
    ///
    /// Exposed so integrity tests can tamper with a store the way an attacker
    /// with disk access would, which is the only way to prove
    /// [`Store::verify`] actually detects tampering. Production code has no
    /// legitimate reason to call this: everything else goes through
    /// [`Store::append`], which chains and signs.
    pub fn records_mut(&mut self) -> &mut Vec<Record> {
        &mut self.records
    }

    /// Seals the unsynced records into a sync batch with a Merkle root.
    pub fn pending_batch(&self) -> Option<Batch> {
        let pending: Vec<&Record> = self.records.iter().filter(|r| !r.synced).collect();
        if pending.is_empty() {
            return None;
        }
        Some(Batch::new(self.identity.clone(), pending.into_iter().cloned().collect()))
    }

    /// Marks a batch's records as acknowledged by the server.
    ///
    /// Rejects an empty batch id list and ids not held by this store, so a
    /// misdirected acknowledgement cannot silently mark nothing.
    pub fn mark_synced(&mut self, ids: &[RecordId]) -> Result<usize, StoreError> {
        let mut marked = 0;
        for id in ids {
            let Some(record) = self.records.iter_mut().find(|r| &r.record_id == id) else {
                return Err(StoreError::Malformed(id.to_string()));
            };
            if !record.synced {
                record.synced = true;
                marked += 1;
            }
        }
        Ok(marked)
    }

    /// Current sync state summary.
    pub fn sync_state(&self) -> SyncState {
        let total = self.records.len();
        let synced = self.records.iter().filter(|r| r.synced).count();
        SyncState { total, synced, pending: total - synced }
    }

    /// Associated data binding a ciphertext to its record and station.
    ///
    /// Including the instance id and record id means a ciphertext lifted from
    /// one station or one record fails to decrypt anywhere else.
    fn aad(&self, data_item: &str, data_class: &str, record_id: Option<RecordId>) -> String {
        format!(
            "eel-bvr-store|v1|{}|{}|{}|{}",
            self.identity.instance_id,
            data_item,
            data_class,
            record_id.map(|id| id.to_string()).unwrap_or_else(|| "new".to_owned()),
        )
    }
}

/// Serialised form of a store, for persisting across restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredStore {
    /// Station identity.
    pub identity: InstanceIdentity,
    /// Base64 data key. Only for the dev keystore; a production store keeps
    /// this in a TPM or HSM and persists a wrapped key instead.
    pub data_key_b64: String,
    /// Records in capture order.
    pub records: Vec<Record>,
}
