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
    /// A filesystem operation failed.
    #[error("store I/O failed: {0}")]
    Io(String),
    /// A value could not be serialised or deserialised.
    #[error("store serialisation failed: {0}")]
    Serialisation(String),
    /// The data key could not be opened under the supplied key-encryption key.
    #[error("store could not be opened: {0}")]
    KeyUnwrap(String),
    /// The file is a store format this build does not understand.
    #[error("unsupported store format {got}, expected {expected}")]
    UnsupportedFormat {
        /// The version found.
        got: u32,
        /// The version this build understands.
        expected: u32,
    },
    /// A store read from disk failed its integrity chain, so it will not be
    /// reopened.
    #[error("store on disk failed its integrity check and will not be reopened: {0}")]
    OnDiskTampered(String),
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

/// On-disk form of a store.
///
/// The data key is **never** written here in the clear. It is sealed under a
/// key-encryption key that the caller supplies at open time, and in a deployed
/// station that KEK lives in a TPM or HSM and never leaves it. Persisting the
/// DEK in the file would make the record encryption pointless, since the file
/// would carry both the key and the ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredStore {
    /// Store format version, so a future migration has somewhere to go.
    pub version: u32,
    /// Station identity. Public, so an auditor can read who wrote the file.
    pub identity: InstanceIdentity,
    /// The data key, sealed under the KEK. Base64.
    pub wrapped_data_key_b64: String,
    /// Every record, sealed as one blob under the data key. Base64.
    pub sealed_records_b64: String,
}

/// Current on-disk format version.
pub const STORE_FORMAT_VERSION: u32 = 1;

impl Store {
    /// Writes the store to `path`, atomically.
    ///
    /// The whole file is written to a sibling temporary path and then renamed
    /// over the target, so a crash or power loss mid-write leaves the previous
    /// store intact rather than a truncated one. On a station that may be
    /// unplugged at any moment, a partial write that destroys completed
    /// registrations is the worst available outcome.
    ///
    /// `kek` seals the data key. It is not written to disk.
    pub fn seal_to_disk(&self, path: &std::path::Path, kek: &[u8; 32]) -> Result<(), StoreError> {
        let records = serde_json::to_vec(&self.records)
            .map_err(|e| StoreError::Serialisation(e.to_string()))?;
        let sealed_records = aead::seal(&self.data_key, &records, AAD_STORE.as_bytes())?;
        let wrapped = aead::seal(kek, &self.data_key, AAD_STORE.as_bytes())?;

        let stored = StoredStore {
            version: STORE_FORMAT_VERSION,
            identity: self.identity.clone(),
            wrapped_data_key_b64: b64(&wrapped),
            sealed_records_b64: b64(&sealed_records),
        };
        let text = serde_json::to_string_pretty(&stored)
            .map_err(|e| StoreError::Serialisation(e.to_string()))?;

        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, text.as_bytes())
            .map_err(|e| StoreError::Io(e.to_string()))?;
        // rename(2) is atomic within a filesystem, so a reader sees either the
        // old file or the new one, never a half-written one.
        std::fs::rename(&temporary, path).map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    /// Reads a store from `path`, unwrapping the data key with `kek`.
    ///
    /// Returns the parts, not a [`Store`], because the station signing key
    /// cannot be recovered from the file: it belongs in a TPM or HSM and is
    /// never written to disk. Pass the result to [`OpenedStore::reopen`] with
    /// a keystore that already holds that key.
    pub fn open_from_disk(path: &std::path::Path, kek: &[u8; 32]) -> Result<OpenedStore, StoreError> {
        let text = std::fs::read_to_string(path).map_err(|e| StoreError::Io(e.to_string()))?;
        let stored: StoredStore =
            serde_json::from_str(&text).map_err(|e| StoreError::Serialisation(e.to_string()))?;

        if stored.version != STORE_FORMAT_VERSION {
            return Err(StoreError::UnsupportedFormat {
                got: stored.version,
                expected: STORE_FORMAT_VERSION,
            });
        }

        let wrapped = unb64(&stored.wrapped_data_key_b64)?;
        let unwrapped =
            aead::open(kek, &wrapped, AAD_STORE.as_bytes()).map_err(|_| {
                StoreError::KeyUnwrap("data key did not open under the supplied KEK".to_owned())
            })?;
        let mut data_key = [0u8; 32];
        data_key.copy_from_slice(&unwrapped);

        let sealed = unb64(&stored.sealed_records_b64)?;
        let plaintext = aead::open(&data_key, &sealed, AAD_STORE.as_bytes()).map_err(|_| {
            StoreError::KeyUnwrap("record blob did not open under the data key".to_owned())
        })?;
        let records: Vec<Record> = serde_json::from_slice(&plaintext)
            .map_err(|e| StoreError::Serialisation(e.to_string()))?;

        Ok(OpenedStore { identity: stored.identity, records, data_key })
    }
}

/// A store read from disk, awaiting the station signing key.
///
/// Splitting the reopen lets the caller supply a TPM- or HSM-backed keystore
/// without the file ever having contained the key.
#[derive(Debug, Clone)]
pub struct OpenedStore {
    identity: InstanceIdentity,
    records: Vec<Record>,
    data_key: [u8; 32],
}

impl OpenedStore {
    /// The station identity read from the file.
    pub fn identity(&self) -> &InstanceIdentity {
        &self.identity
    }

    /// How many records the file held.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the file held no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Rebuilds a usable [`Store`], verifying the integrity chain first.
    ///
    /// Verification happens here rather than being left to the caller, so a
    /// store that fails its chain is never handed out. A tampered file yields
    /// [`StoreError::OnDiskTampered`], not a store.
    pub fn reopen(self, keys: SoftwareKeyStore) -> Result<Store, StoreError> {
        let store = Store { identity: self.identity, keys, records: self.records, data_key: self.data_key };
        store.verify().map_err(|e| StoreError::OnDiskTampered(e.to_string()))?;
        Ok(store)
    }
}

impl Store {
    /// The data key, for tests that need to assert it is not written in the
    /// clear. Deliberately named for a test so production code has no
    /// reason to call it.
    pub fn data_key_for_test(&self) -> [u8; 32] {
        self.data_key
    }
}

/// Associated data for the whole-store seal, distinct from the per-record one.
const AAD_STORE: &str = "eel-bvr-store-file|v1";

/// Base64-encodes bytes.
fn b64(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

/// Decodes base64, mapping failure to a store error.
fn unb64(text: &str) -> Result<Vec<u8>, StoreError> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, text)
        .map_err(|e| StoreError::Serialisation(format!("malformed base64: {e}")))
}
