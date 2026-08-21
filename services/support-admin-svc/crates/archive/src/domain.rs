use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionStatus {
    Ingest,
    Sealed,
    Attested,
    Public,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestStatus {
    Pending,
    Running,
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    ReadOnly,
    Download,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

// ---------------------------------------------------------------------------
// Aggregates
// ---------------------------------------------------------------------------

/// Long-term archival collection tied to an election.
///
/// Retention period: 15 years from creation. Computed as expires_at = created_at + 15 years.
///
/// WORM note: MinIO Object Lock is configured via bucket policy (COMPLIANCE mode,
/// 15-year retention duration). The storage layer enforces immutability; this domain
/// model does not replicate that constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveCollection {
    pub id: Uuid,
    pub election_id: Uuid,
    pub collection_name: String,
    /// Module identifiers whose data is included in this collection (e.g. "budget", "payroll").
    pub source_modules: Vec<String>,
    /// SHA-256 hex digest of the sealed BagIt bag.
    pub bag_checksum: Option<String>,
    /// BagIt spec version string (e.g. "1.0").
    pub bagit_version: String,
    /// DataCite DOI once deposited.
    pub doi: Option<String>,
    pub status: CollectionStatus,
    pub created_at: DateTime<Utc>,
}

impl ArchiveCollection {
    pub fn new(
        election_id: Uuid,
        collection_name: impl Into<String>,
        source_modules: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            election_id,
            collection_name: collection_name.into(),
            source_modules,
            bag_checksum: None,
            bagit_version: "1.0".to_string(),
            doi: None,
            status: CollectionStatus::Ingest,
            created_at: Utc::now(),
        }
    }

    /// Computed 15-year retention expiry.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.created_at + Duration::days(365 * 15)
    }

    /// Transition to Sealed. Valid only from Ingest or Attested.
    pub fn seal(&mut self) -> Result<(), DomainError> {
        match self.status {
            CollectionStatus::Ingest | CollectionStatus::Attested => {
                self.status = CollectionStatus::Sealed;
                Ok(())
            }
            _ => Err(DomainError::InvalidTransition {
                detail: format!("cannot seal a collection in {:?} status", self.status),
            }),
        }
    }
}

/// Tracks ingestion of a source module's data into a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestJob {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub source_module: String,
    pub source_ref: String,
    pub items_expected: u32,
    pub items_ingested: u32,
    pub status: IngestStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl IngestJob {
    pub fn new(
        collection_id: Uuid,
        source_module: impl Into<String>,
        source_ref: impl Into<String>,
        items_expected: u32,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            collection_id,
            source_module: source_module.into(),
            source_ref: source_ref.into(),
            items_expected,
            items_ingested: 0,
            status: IngestStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Cryptographic preservation attestation for a collection.
///
/// The attestation_hash binds: collection_id + fixity_check_result + attested_by.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationAttestation {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub attested_by: Uuid,
    pub fixity_check_result: String,
    pub fixity_check_at: DateTime<Utc>,
    pub attestation_hash: String,
}

impl PreservationAttestation {
    pub fn new(
        collection_id: Uuid,
        attested_by: Uuid,
        fixity_check_result: impl Into<String>,
    ) -> Self {
        let result = fixity_check_result.into();
        let hash = Self::compute_hash(collection_id, &result, attested_by);
        Self {
            id: Uuid::now_v7(),
            collection_id,
            attested_by,
            fixity_check_result: result,
            fixity_check_at: Utc::now(),
            attestation_hash: hash,
        }
    }

    /// SHA-256 of (collection_id as bytes) || fixity_check_result bytes || (attested_by as bytes).
    pub fn compute_hash(collection_id: Uuid, fixity_result: &str, attested_by: Uuid) -> String {
        let mut hasher = Sha256::new();
        hasher.update(collection_id.as_bytes());
        hasher.update(fixity_result.as_bytes());
        hasher.update(attested_by.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Research access request for a sealed/public collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchAccessRequest {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub researcher_id: Uuid,
    pub institution: String,
    pub purpose: String,
    pub access_level: AccessLevel,
    pub expires_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
    pub status: AccessStatus,
}

impl ResearchAccessRequest {
    pub fn new(
        collection_id: Uuid,
        researcher_id: Uuid,
        institution: impl Into<String>,
        purpose: impl Into<String>,
        access_level: AccessLevel,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            collection_id,
            researcher_id,
            institution: institution.into(),
            purpose: purpose.into(),
            access_level,
            expires_at,
            granted_by: None,
            status: AccessStatus::Pending,
        }
    }
}

/// Certificate proving destruction of a collection after retention period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionCertificate {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub destruction_basis: String,
    pub destroyed_at: DateTime<Utc>,
    pub witnessed_by: Uuid,
    pub certificate_hash: String,
}

impl DestructionCertificate {
    pub fn new(
        collection_id: Uuid,
        destruction_basis: impl Into<String>,
        witnessed_by: Uuid,
    ) -> Self {
        let basis = destruction_basis.into();
        let now = Utc::now();
        let hash = Self::compute_hash(collection_id, &basis, witnessed_by, now);
        Self {
            id: Uuid::now_v7(),
            collection_id,
            destruction_basis: basis,
            destroyed_at: now,
            witnessed_by,
            certificate_hash: hash,
        }
    }

    fn compute_hash(
        collection_id: Uuid,
        basis: &str,
        witnessed_by: Uuid,
        destroyed_at: DateTime<Utc>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(collection_id.as_bytes());
        hasher.update(basis.as_bytes());
        hasher.update(witnessed_by.as_bytes());
        hasher.update(destroyed_at.to_rfc3339().as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// DataCite citation record for a publicly deposited collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRecord {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub doi: String,
    pub datacite_ref: String,
    pub deposited_at: DateTime<Utc>,
    pub citation_text: String,
}

impl CitationRecord {
    pub fn new(
        collection_id: Uuid,
        doi: impl Into<String>,
        datacite_ref: impl Into<String>,
        citation_text: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            collection_id,
            doi: doi.into(),
            datacite_ref: datacite_ref.into(),
            deposited_at: Utc::now(),
            citation_text: citation_text.into(),
        }
    }
}
