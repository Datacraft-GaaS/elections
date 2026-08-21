use archive::domain::{
    AccessLevel, ArchiveCollection, CollectionStatus, DestructionCertificate, IngestJob,
    PreservationAttestation, ResearchAccessRequest,
};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn collection_new_has_ingest_status() {
    let c = ArchiveCollection::new(
        Uuid::now_v7(),
        "Test Collection",
        vec!["budget".to_string()],
    );
    assert_eq!(c.status, CollectionStatus::Ingest);
    assert_eq!(c.bagit_version, "1.0");
}

#[test]
fn collection_seal_from_ingest_succeeds() {
    let mut c = ArchiveCollection::new(
        Uuid::now_v7(),
        "Test Collection",
        vec!["budget".to_string()],
    );
    c.seal().expect("seal should succeed from Ingest");
    assert_eq!(c.status, CollectionStatus::Sealed);
}

#[test]
fn collection_seal_from_sealed_fails() {
    let mut c = ArchiveCollection::new(
        Uuid::now_v7(),
        "Test Collection",
        vec!["budget".to_string()],
    );
    c.seal().unwrap();
    let err = c.seal().expect_err("double-seal should fail");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("sealed"),
        "expected 'sealed' in: {msg}"
    );
}

#[test]
fn collection_expires_at_is_15_years_later() {
    let c = ArchiveCollection::new(
        Uuid::now_v7(),
        "Test Collection",
        vec!["payroll".to_string()],
    );
    let diff = c.expires_at() - c.created_at;
    // 15 years * 365 days
    assert_eq!(diff.num_days(), 365 * 15);
}

#[test]
fn ingest_job_new_is_pending() {
    let job = IngestJob::new(Uuid::now_v7(), "budget", "ref-001", 100);
    assert_eq!(job.items_ingested, 0);
    assert_eq!(job.items_expected, 100);
}

#[test]
fn attestation_hash_is_deterministic() {
    let cid = Uuid::now_v7();
    let aid = Uuid::now_v7();
    let h1 = PreservationAttestation::compute_hash(cid, "ok", aid);
    let h2 = PreservationAttestation::compute_hash(cid, "ok", aid);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64); // SHA-256 hex
}

#[test]
fn attestation_hash_changes_on_different_inputs() {
    let cid = Uuid::now_v7();
    let aid = Uuid::now_v7();
    let h1 = PreservationAttestation::compute_hash(cid, "ok", aid);
    let h2 = PreservationAttestation::compute_hash(cid, "failed", aid);
    assert_ne!(h1, h2);
}

#[test]
fn research_access_request_is_pending() {
    let req = ResearchAccessRequest::new(
        Uuid::now_v7(),
        Uuid::now_v7(),
        "University of Nairobi",
        "Electoral integrity research",
        AccessLevel::ReadOnly,
        Utc::now() + chrono::Duration::days(30),
    );
    assert!(req.granted_by.is_none());
    assert_eq!(req.access_level, AccessLevel::ReadOnly);
}

#[test]
fn destruction_certificate_has_hash() {
    let cert = DestructionCertificate::new(
        Uuid::now_v7(),
        "Retention period expired per schedule 3",
        Uuid::now_v7(),
    );
    assert_eq!(cert.certificate_hash.len(), 64);
}
