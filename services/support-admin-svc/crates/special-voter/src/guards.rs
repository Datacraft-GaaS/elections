//! s.44A override guardrails: dual sign-off and IAB 5% sample audit.
//! Elections Act s.44A; FR-M25-012, AC-M25-08, FR-M25-021.

use thiserror::Error;
use uuid::Uuid;

/// Result of a dual-officer s.44A override attempt.
#[derive(Debug, Clone)]
pub struct S44AOverrideRequest {
    pub facility_id: Uuid,
    pub reason_code: String,
    pub requesting_officer_id: String,
    pub countersigning_officer_id: String,
}

#[derive(Debug, Error)]
pub enum GuardError {
    #[error(
        "s.44A dual sign-off: requesting and countersigning officer must be different individuals"
    )]
    SameOfficer,
    #[error("s.44A dual sign-off: both officer IDs must be non-empty")]
    MissingOfficer,
    #[error("IAB 5% audit: sample_fraction must be >= 0.05 (got {got:.3})")]
    InsufficientAuditFraction { got: f64 },
}

/// Validate s.44A dual sign-off requirement.
/// Both officers must be present and distinct (AC-M25-08).
pub fn validate_s44a_dual_signoff(req: &S44AOverrideRequest) -> Result<(), GuardError> {
    if req.requesting_officer_id.is_empty() || req.countersigning_officer_id.is_empty() {
        return Err(GuardError::MissingOfficer);
    }
    if req.requesting_officer_id == req.countersigning_officer_id {
        return Err(GuardError::SameOfficer);
    }
    Ok(())
}

/// Confirm IAB sample fraction meets the 5% floor (FR-M25-008, FR-M25-021).
pub fn validate_iab_sample_fraction(sampled: u32, total: u32) -> Result<(), GuardError> {
    if total == 0 {
        return Ok(());
    }
    let fraction = sampled as f64 / total as f64;
    if fraction < 0.05 {
        return Err(GuardError::InsufficientAuditFraction { got: fraction });
    }
    Ok(())
}

/// Deterministic sample selector: reproducible from Merkle chain seed.
/// Returns selected stream IDs at the 5% floor from the given pool.
pub fn select_iab_sample(pool: &[Uuid], seed_hex: &str) -> Vec<Uuid> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let target = ((pool.len() as f64) * 0.05).ceil() as usize;
    let mut indexed: Vec<(u64, Uuid)> = pool
        .iter()
        .map(|id| {
            let mut h = DefaultHasher::new();
            seed_hex.hash(&mut h);
            id.hash(&mut h);
            (h.finish(), *id)
        })
        .collect();
    indexed.sort_by_key(|(score, _)| *score);
    indexed.into_iter().take(target).map(|(_, id)| id).collect()
}
