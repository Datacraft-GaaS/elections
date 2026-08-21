//! Bundle domain types (WP-4.6, M14).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::ReproError;
use crate::manifest::BundleManifest;

/// Component type discriminant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Forms,
    Tokens,
    Src,
    Build,
    Sbom,
}

/// One component of a reproducibility bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleComponent {
    pub comp_id: Uuid,
    pub bundle_id: Uuid,
    pub component_type: ComponentType,
    pub reference: String,
    pub sha256: String,
    pub size_bytes: u64,
}

impl BundleComponent {
    pub fn new(
        bundle_id: Uuid,
        component_type: ComponentType,
        reference: String,
        data: &[u8],
    ) -> Self {
        let digest = Sha256::digest(data);
        let sha256 = hex::encode(digest);
        Self {
            comp_id: Uuid::new_v4(),
            bundle_id,
            component_type,
            reference,
            sha256,
            size_bytes: data.len() as u64,
        }
    }
}

/// An assembled bundle (in-memory representation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub bundle_id: Uuid,
    pub election_id: String,
    pub manifest: BundleManifest,
    pub assembled_at: DateTime<Utc>,
    pub size_bytes: u64,
}

impl Bundle {
    /// Verify all component hashes in the manifest match provided bytes via `fetch`.
    pub fn verify_components<F>(&self, fetch: F) -> Result<(), ReproError>
    where
        F: Fn(&str) -> Option<Vec<u8>>,
    {
        for entry in &self.manifest.components {
            let data = fetch(&entry.reference)
                .ok_or_else(|| ReproError::ComponentMissing(entry.reference.clone()))?;
            let digest = hex::encode(Sha256::digest(&data));
            if digest != entry.sha256 {
                return Err(ReproError::HashMismatch {
                    expected: entry.sha256.clone(),
                    actual: digest,
                });
            }
        }
        Ok(())
    }
}
