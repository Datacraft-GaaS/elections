//! Bundle manifest types (WP-4.6, M14).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Manifest format version (semver string).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestVersion(pub String);

impl Default for ManifestVersion {
    fn default() -> Self {
        Self("1.0.0".into())
    }
}

/// Root manifest embedded in every bundle archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub format_version: ManifestVersion,
    pub bundle_id: Uuid,
    pub election_id: String,
    pub assembled_at: DateTime<Utc>,
    pub merkle_root: String,
    pub components: Vec<ComponentEntry>,
    /// SHA-256 of the deterministic build log.
    pub build_log_hash: String,
    /// Reference to a prior version (correction bundles only).
    pub prior_bundle_id: Option<Uuid>,
    /// Ed25519 hex signature over the canonical manifest bytes (minus this field).
    pub signature: Option<String>,
}

impl BundleManifest {
    /// Canonical bytes for signing: manifest serialised without the `signature` field.
    pub fn canonical_bytes(&self) -> serde_json::Result<Vec<u8>> {
        let mut v = serde_json::to_value(self)?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("signature");
        }
        serde_json::to_vec(&v)
    }
}

/// One component entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEntry {
    pub comp_id: Uuid,
    pub component_type: String,
    pub reference: String,
    pub sha256: String,
    pub size_bytes: u64,
}
