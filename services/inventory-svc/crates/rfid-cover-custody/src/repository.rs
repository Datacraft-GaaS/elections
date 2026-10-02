//! sqlx runtime queries for RFID cover custody tables.
//!
//! All queries use `sqlx::query()` / `sqlx::query_as()` at runtime; no compile-time
//! DATABASE_URL is required.

use chrono::Utc;
use sqlx::PgPool;
use ed25519_dalek::VerifyingKey;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{CoverRead, CoverState, CoverTag, CoverTamperAlert, ReaderType, Severity, TamperState};
use crate::error::RfidError;

// ---------------------------------------------------------------------------
// CoverTag
// ---------------------------------------------------------------------------

/// Insert a new cover tag row.
pub async fn insert_tag(pool: &PgPool, tag: &CoverTag) -> Result<(), RfidError> {
    sqlx::query(
        r#"
        INSERT INTO eel_rfid_cover_tags
            (tag_id, epc, manufacturer, lot_id, chip_family,
             tag_secret_hsm_key_id, issued_at, tamper_state)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(tag.tag_id)
    .bind(&tag.epc)
    .bind(&tag.manufacturer)
    .bind(tag.lot_id)
    .bind(&tag.chip_family)
    .bind(&tag.tag_secret_hsm_key_id)
    .bind(tag.issued_at)
    .bind(tag.tamper_state.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch a tag by ID.
pub async fn fetch_tag(pool: &PgPool, tag_id: Uuid) -> Result<Option<CoverTag>, RfidError> {
    let row = sqlx::query(
        r#"
        SELECT tag_id, epc, manufacturer, lot_id, chip_family,
               tag_secret_hsm_key_id, issued_at, paired_with_box, paired_at,
               tamper_state, tamper_detected_at, tamper_detected_by, retired_at
        FROM eel_rfid_cover_tags
        WHERE tag_id = $1
        "#,
    )
    .bind(tag_id)
    .fetch_optional(pool)
    .await
    .map_err(RfidError::Database)?;

    use sqlx::Row;
    Ok(row.map(|r: sqlx::postgres::PgRow| CoverTag {
        tag_id: r.get("tag_id"),
        epc: r.get("epc"),
        manufacturer: r.get("manufacturer"),
        lot_id: r.get("lot_id"),
        chip_family: r.get("chip_family"),
        tag_secret_hsm_key_id: r.get("tag_secret_hsm_key_id"),
        issued_at: r.get("issued_at"),
        paired_with_box: r.get("paired_with_box"),
        paired_at: r.get("paired_at"),
        tamper_state: TamperState::from_str_lossy(r.get::<String, _>("tamper_state").as_str()),
        tamper_detected_at: r.get("tamper_detected_at"),
        tamper_detected_by: r.get("tamper_detected_by"),
        retired_at: r.get("retired_at"),
    }))
}

/// Update pairing fields on a tag.
pub async fn update_tag_pairing(
    pool: &PgPool,
    tag_id: Uuid,
    box_body_id: Uuid,
) -> Result<(), RfidError> {
    sqlx::query(
        "UPDATE eel_rfid_cover_tags SET paired_with_box = $2, paired_at = $3 WHERE tag_id = $1",
    )
    .bind(tag_id)
    .bind(box_body_id)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CoverRead
// ---------------------------------------------------------------------------

/// Insert a new cover read row.
pub async fn insert_read(pool: &PgPool, read: &CoverRead) -> Result<(), RfidError> {
    sqlx::query(
        r#"
        INSERT INTO eel_rfid_cover_reads
            (read_id, tag_id, reader_id, reader_type, read_at,
             geo_lat, geo_lon, geo_accuracy_m,
             tag_challenge, tag_response, challenge_valid, tamper_indicator,
             reader_signature, custody_event_id, correlation_id)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
        "#,
    )
    .bind(read.read_id)
    .bind(read.tag_id)
    .bind(&read.reader_id)
    .bind(read.reader_type.as_str())
    .bind(read.read_at)
    .bind(read.geo_lat)
    .bind(read.geo_lon)
    .bind(read.geo_accuracy_m)
    .bind(read.tag_challenge.as_deref())
    .bind(read.tag_response.as_deref())
    .bind(read.challenge_valid)
    .bind(read.tamper_indicator)
    .bind(read.reader_signature.as_deref())
    .bind(read.custody_event_id)
    .bind(read.correlation_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// CoverStateTransition
// ---------------------------------------------------------------------------

/// Officer key registry, parsed once from `EEL_CUSTODY_OFFICER_KEYS`.
fn officer_registry() -> &'static HashMap<String, VerifyingKey> {
    static REGISTRY: std::sync::OnceLock<HashMap<String, VerifyingKey>> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let spec = std::env::var("EEL_CUSTODY_OFFICER_KEYS").unwrap_or_default();
        match crate::officer_sig::parse_registry(&spec) {
            Ok(map) => map,
            Err(e) => {
                tracing::error!(error = %e, "EEL_CUSTODY_OFFICER_KEYS malformed; high-risk custody transitions will be REJECTED");
                HashMap::new()
            }
        }
    })
}

/// Insert a new state transition row.
///
/// High-risk transitions (SEALED_RETURN / TALLY_ARRIVED / COUNT_COMPLETE /
/// STORAGE_ARCHIVE) are refused unless they carry an Ed25519 officer
/// signature that verifies over `this_hash` against a registered officer key.
pub async fn insert_transition(
    pool: &PgPool,
    t: &crate::domain::CoverStateTransition,
) -> Result<(), RfidError> {
    use crate::officer_sig::{check_transition_signature, SignatureCheck};
    match check_transition_signature(
        officer_registry(),
        t.officer_id,
        t.to_state.as_str(),
        &t.this_hash,
        t.officer_signature.as_deref(),
    ) {
        SignatureCheck::Verified | SignatureCheck::NotRequired => {}
        SignatureCheck::Missing => {
            return Err(RfidError::OfficerSignatureRequired {
                to_state: t.to_state.as_str().to_string(),
            })
        }
        SignatureCheck::UnregisteredOfficer => {
            return Err(RfidError::OfficerUnregistered {
                officer_id: t.officer_id.map(|o| o.to_string()).unwrap_or_default(),
            })
        }
        SignatureCheck::Invalid => return Err(RfidError::OfficerSignatureInvalid),
    }

    sqlx::query(
        r#"
        INSERT INTO eel_cover_state_transitions
            (transition_id, tag_id, from_state, to_state,
             triggered_by_read, officer_id, officer_svid, officer_signature,
             prev_hash, this_hash, transitioned_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(t.transition_id)
    .bind(t.tag_id)
    .bind(t.from_state.as_str())
    .bind(t.to_state.as_str())
    .bind(t.triggered_by_read)
    .bind(t.officer_id)
    .bind(t.officer_svid.as_deref())
    .bind(t.officer_signature.as_deref())
    .bind(t.prev_hash.as_deref())
    .bind(&t.this_hash)
    .bind(t.transitioned_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch all transitions for a tag in ascending time order.
pub async fn fetch_transitions(
    pool: &PgPool,
    tag_id: Uuid,
) -> Result<Vec<crate::domain::CoverStateTransition>, RfidError> {
    let rows = sqlx::query(
        r#"
        SELECT transition_id, tag_id, from_state, to_state,
               triggered_by_read, officer_id, officer_svid, officer_signature,
               prev_hash, this_hash, transitioned_at
        FROM eel_cover_state_transitions
        WHERE tag_id = $1
        ORDER BY transitioned_at ASC
        "#,
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    rows.into_iter()
        .map(|r| {
            let from_str: String = r.try_get("from_state")?;
            let to_str: String = r.try_get("to_state")?;
            Ok(crate::domain::CoverStateTransition {
                transition_id: r.try_get("transition_id")?,
                tag_id: r.try_get("tag_id")?,
                from_state: CoverState::from_str(&from_str)
                    .ok_or_else(|| sqlx::Error::Decode(format!("unknown from_state: {from_str}").into()))?,
                to_state: CoverState::from_str(&to_str)
                    .ok_or_else(|| sqlx::Error::Decode(format!("unknown to_state: {to_str}").into()))?,
                triggered_by_read: r.try_get("triggered_by_read")?,
                officer_id: r.try_get("officer_id")?,
                officer_svid: r.try_get("officer_svid")?,
                officer_signature: r.try_get("officer_signature")?,
                prev_hash: r.try_get("prev_hash")?,
                this_hash: r.try_get("this_hash")?,
                transitioned_at: r.try_get("transitioned_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(RfidError::Database)
}

// ---------------------------------------------------------------------------
// CoverTamperAlert
// ---------------------------------------------------------------------------

/// Insert a new tamper alert row.
pub async fn insert_alert(pool: &PgPool, alert: &CoverTamperAlert) -> Result<(), RfidError> {
    sqlx::query(
        r#"
        INSERT INTO eel_cover_tamper_alerts
            (alert_id, tag_id, detected_at, triggering_read,
             expected_state, observed_state, severity, ecc_dispatched)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(alert.alert_id)
    .bind(alert.tag_id)
    .bind(alert.detected_at)
    .bind(alert.triggering_read)
    .bind(alert.expected_state.as_deref())
    .bind(alert.observed_state.as_deref())
    .bind(alert.severity.as_str())
    .bind(alert.ecc_dispatched)
    .execute(pool)
    .await?;

    Ok(())
}

/// Pending (unresolved) tamper alerts, optionally narrowed by severity.
///
/// The `status` query parameter narrows by **severity**, which is the field its
/// OpenAPI description names. `resolved_at IS NULL` is what makes an alert pending
/// and is not negotiable: this endpoint is the pending queue, so a caller asking
/// for resolved alerts gets the unresolved ones rather than the other way round.
pub async fn fetch_pending_alerts(
    pool: &PgPool,
    status: Option<&str>,
) -> Result<Vec<CoverTamperAlert>, RfidError> {
    let rows = match status.map(str::trim).filter(|s| !s.is_empty()) {
        Some(severity) => {
            sqlx::query(
                r#"
                SELECT alert_id, tag_id, detected_at, triggering_read,
                       expected_state, observed_state, severity, ecc_dispatched,
                       ecc_dispatched_at, resolved_at, resolution_note
                FROM eel_cover_tamper_alerts
                WHERE resolved_at IS NULL AND severity = $1
                ORDER BY detected_at DESC
                "#,
            )
            .bind(severity)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query(
                r#"
                SELECT alert_id, tag_id, detected_at, triggering_read,
                       expected_state, observed_state, severity, ecc_dispatched,
                       ecc_dispatched_at, resolved_at, resolution_note
                FROM eel_cover_tamper_alerts
                WHERE resolved_at IS NULL
                ORDER BY detected_at DESC
                "#,
            )
            .fetch_all(pool)
            .await?
        }
    };

    use sqlx::Row;
    rows.into_iter()
        .map(|r| {
            let severity_str: String = r.try_get("severity")?;
            Ok(CoverTamperAlert {
                alert_id: r.try_get("alert_id")?,
                tag_id: r.try_get("tag_id")?,
                detected_at: r.try_get("detected_at")?,
                triggering_read: r.try_get("triggering_read")?,
                expected_state: r.try_get("expected_state")?,
                observed_state: r.try_get("observed_state")?,
                severity: Severity::from_str_lossy(&severity_str),
                ecc_dispatched: r.try_get("ecc_dispatched")?,
                ecc_dispatched_at: r.try_get("ecc_dispatched_at")?,
                resolved_at: r.try_get("resolved_at")?,
                resolution_note: r.try_get("resolution_note")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(RfidError::Database)
}

/// Mark an alert resolved.
pub async fn resolve_alert(
    pool: &PgPool,
    alert_id: Uuid,
    note: &str,
) -> Result<(), RfidError> {
    sqlx::query(
        "UPDATE eel_cover_tamper_alerts SET resolved_at = $2, resolution_note = $3 WHERE alert_id = $1",
    )
    .bind(alert_id)
    .bind(Utc::now())
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helper parse-lossy methods on domain enums
// ---------------------------------------------------------------------------

impl TamperState {
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "intact" => Self::Intact,
            "cover_lifted" => Self::CoverLifted,
            "antenna_broken" => Self::AntennaBroken,
            "clone_detected" => Self::CloneDetected,
            "killed" => Self::Killed,
            _ => Self::Intact,
        }
    }
}

impl Severity {
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "critical" => Self::Critical,
            "warn" => Self::Warn,
            _ => Self::Info,
        }
    }
}

impl ReaderType {
    pub fn from_str_lossy(s: &str) -> Self {
        ReaderType::from_str(s).unwrap_or(ReaderType::KitHandheld)
    }
}
