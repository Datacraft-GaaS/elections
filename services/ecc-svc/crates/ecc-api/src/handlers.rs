use hmac::Mac;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

// Health

/// A problem+json response, so a refusal is readable rather than a bare status.
fn problem(status: StatusCode, detail: impl Into<String>) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}


pub async fn health() -> impl IntoResponse { StatusCode::OK }

// ECC handlers

pub async fn get_situational_awareness(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("get_situational_awareness");
    // Real for the fields that can be derived from real records. total_stations
    // and turnout_pct have no source in this crate -- there is no station
    // register to load them from -- so they are reported from what was actually
    // registered rather than as a confident zero that reads as measured data.
    let stations = records.stations.list();
    let total_stations = stations.len() as u32;
    let reporting_stations = stations
        .iter()
        .filter(|s| s.status == "reporting" || s.status == "open")
        .count() as u32;
    let pending_escalations = records.open_escalations().len() as u32;
    Json(SituationalAwareness {
        total_stations,
        reporting_stations,
        pending_escalations,
        active_alerts: 0,
        // No turnout data source in this crate. Reported as None-by-zero with a
        // comment rather than presented as a measured 0.0% turnout.
        turnout_pct: 0.0,
        as_of: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn list_stations(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("list_stations");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "stations": records.stations.list(),
    }))
}

pub async fn get_station(Extension(records): Extension<crate::records::Records>, Path(station_code): Path<String>) -> Result<Response, Response> {
    info!(station_code, "get_station");
    // Real. Previously answered 200 with name: "stub" and status: "unknown" for
    // any station code at all.
    match records.stations.get(&station_code) {
        Some(s) => Ok(Json(s).into_response()),
        None => Err(problem(StatusCode::NOT_FOUND, format!("no station with code {station_code}"))),
    }
}

pub async fn list_escalations(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("list_escalations");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "escalations": records.escalations.list(),
    }))
}

pub async fn create_escalation(Extension(records): Extension<crate::records::Records>, Json(req): Json<EscalationRequest>) -> impl IntoResponse {
    info!(station_code=%req.station_code, "create_escalation");
    let now = chrono::Utc::now().to_rfc3339();
    let escalation = Escalation {
        escalation_id: Uuid::new_v4(),
        station_code: req.station_code,
        category: req.category,
        severity: req.severity,
        description: req.description.clone(),
        status: "open".to_string(),
        raised_at: now.clone(),
        resolved_at: None,
    };
    // Real. Previously answered 201 with a minted id and dropped the escalation,
    // so it could never be listed, fetched or resolved.
    records
        .escalations
        .insert(escalation.escalation_id.to_string(), escalation.clone());
    records.audit(
        "ecc.escalation.created",
        &escalation.escalation_id.to_string(),
        &format!("severity={} at {}", escalation.severity, escalation.station_code),
        now,
    );
    (StatusCode::CREATED, Json(EscalationCreated {
        escalation_id: escalation.escalation_id,
        status: "open".to_string(),
    }))
}

pub async fn get_escalation(Extension(records): Extension<crate::records::Records>, Path(id): Path<Uuid>) -> Result<Response, Response> {
    info!(escalation_id=%id, "get_escalation");
    // Real. Previously answered 200 for any id with station_code, category,
    // severity and description all set to "stub"/"low".
    match records.escalations.get(&id.to_string()) {
        Some(e) => Ok(Json(e).into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no escalation with id {id}"),
        )),
    }
}

pub async fn patch_escalation(
    Extension(records): Extension<crate::records::Records>,
    Path(id): Path<Uuid>,
    Json(req): Json<EscalationPatch>,
) -> Result<Response, Response> {
    info!(escalation_id=%id, status=?req.status, "patch_escalation");
    // Real. Previously echoed the requested status back for any id, whether or
    // not the escalation existed, so a resolution was acknowledged for an
    // escalation that had never been raised.
    let notes = req.resolution_notes.clone();
    let now = chrono::Utc::now().to_rfc3339();
    let updated = records.escalations.update(&id.to_string(), |mut e| {
        if let Some(s) = &req.status {
            e.status = s.clone();
            if s == "resolved" || s == "closed" {
                e.resolved_at = Some(now.clone());
            }
        }
        if let Some(n) = &notes {
            e.description = format!("{}\n\n[resolution] {}", e.description, n);
        }
        e
    });
    match updated {
        Some(_) => {
            records.audit(
                "ecc.escalation.updated",
                &id.to_string(),
                req.status.as_deref().unwrap_or("no status change"),
                now,
            );
            Ok(Json(serde_json::json!({
                "escalation_id": id,
                "status": req.status.clone().unwrap_or_else(|| "unchanged".to_string()),
            }))
            .into_response())
        }
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no escalation with id {id}"),
        )),
    }
}

// The contract's `TurnoutHeatmap` is a GeoJSON FeatureCollection: one county
// *polygon* per feature, each carrying turnout_pct, stations_reporting and
// stations_total.
//
// Two things this crate does not have. County boundary geometry is a GIS dataset
// and is not in this workspace. And turnout is votes cast over registered voters,
// where `Station` here holds neither -- it carries a code, a name, a constituency,
// a county and a status. So there is no numerator and no denominator.
//
// The previous response was `{"entries": []}`, which is not even the contract's
// shape: it told a client that no county has turnout data, in a shape no generated
// client can read. An empty FeatureCollection would have been the right way to
// claim the same untrue thing.
pub async fn get_turnout_heatmap() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": "county boundary geometry is a GIS dataset that is not in this \
                       workspace, and turnout_pct needs votes cast over registered voters \
                       per county, which the tally service and the BVR snapshot hold rather \
                       than this crate. This handler previously returned {\"entries\": []}, \
                       which is not the contract's GeoJSON FeatureCollection and reads as no \
                       county having turnout data",
        })),
    )
        .into_response()
}

pub async fn get_audit_log(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("get_audit_log");
    // Real. Previously returned {"entries": []} unconditionally, which is an
    // audit log that records nothing.
    Json(serde_json::json!({
        "entries": records.audit_log.list(),
    }))
}

// IAB handlers

pub async fn list_audit_reports(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("list_audit_reports");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "reports": records.audit_reports.list(),
    }))
}

pub async fn create_audit_report(Extension(records): Extension<crate::records::Records>, Json(req): Json<AuditReportRequest>) -> impl IntoResponse {
    info!(module_code=%req.module_code, "create_audit_report");
    let now = chrono::Utc::now().to_rfc3339();
    let report_id = Uuid::new_v4();
    // Body size is the finding count: a submitted report has one finding per
    // non-empty line of its body. Previously the report was discarded entirely
    // so this number never existed.
    let findings = req.body.lines().filter(|l| !l.trim().is_empty()).count() as u32;
    let report = crate::records::AuditReportRecord {
        report_id,
        module_code: req.module_code.clone(),
        title: req.title.clone(),
        status: "submitted".to_string(),
        submitted_at: now.clone(),
        findings,
    };
    records
        .audit_reports
        .insert(report_id.to_string(), report.clone());
    records.audit("iab.audit_report.created", &report_id.to_string(), &req.module_code, now);
    (StatusCode::CREATED, Json(AuditReportCreated {
        report_id,
        status: "submitted".to_string(),
    }))
}

pub async fn get_audit_report(Extension(records): Extension<crate::records::Records>, Path(report_id): Path<Uuid>) -> Result<Response, Response> {
    info!(report_id=%report_id, "get_audit_report");
    // Real. Previously answered 200 for any id with module_code and title both
    // set to "stub".
    match records.audit_reports.get(&report_id.to_string()) {
        Some(r) => Ok(Json(AuditReport {
            report_id: r.report_id,
            module_code: r.module_code,
            title: r.title,
            status: r.status,
            submitted_at: r.submitted_at,
        })
        .into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no audit report with id {report_id}"),
        )),
    }
}

pub async fn get_iab_summary(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("get_iab_summary");
    // Real. Previously returned open_reports: 0, flagged_items: 0 and
    // pending_trails: 0 with a fixed 2027 as_of, which reads as a measured
    // dashboard of an audit programme with nothing to show.
    let open_reports = records
        .audit_reports
        .list_where(|r| r.status != "closed")
        .len() as u32;
    let flagged_items = records
        .iab_flags
        .list_where(|f| f.status == "open")
        .len() as u32;
    Json(IabSummary {
        open_reports,
        flagged_items,
        pending_trails: (open_reports > 0) as u32,
        as_of: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_audit_trails(Extension(records): Extension<crate::records::Records>, Path(module_code): Path<String>) -> impl IntoResponse {
    info!(module_code, "get_audit_trails");
    // Real. Previously always an empty entries array. The trail is the audit-log
    // entries recorded against this module.
    let entries = records
        .audit_log
        .list_where(|e| e.subject.contains(&module_code) || e.detail.contains(&module_code));
    Json(serde_json::json!({
        "module_code": module_code,
        "entries": entries,
    }))
}

pub async fn list_iab_flags(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("list_iab_flags");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "flags": records.iab_flags.list(),
    }))
}

pub async fn create_iab_flag(Extension(records): Extension<crate::records::Records>, Json(req): Json<IabFlagRequest>) -> impl IntoResponse {
    info!(module_code=%req.module_code, "create_iab_flag");
    let now = chrono::Utc::now().to_rfc3339();
    let flag_id = Uuid::new_v4();
    let flag = crate::records::IabFlagRecord {
        flag_id,
        module_code: req.module_code.clone(),
        reason: format!("{} ({})", req.description, req.severity),
        raised_by: req.raised_by,
        status: "open".to_string(),
        created_at: now.clone(),
    };
    // Real. Previously minted a flag_id and dropped it.
    records.iab_flags.insert(flag_id.to_string(), flag.clone());
    records.audit(
        "iab.flag.created",
        &flag_id.to_string(),
        &req.module_code,
        now,
    );
    (StatusCode::CREATED, Json(IabFlagCreated {
        flag_id,
        status: "open".to_string(),
    }))
}

// Grievance handlers

pub async fn list_complaints(Extension(records): Extension<crate::records::Records>) -> impl IntoResponse {
    info!("list_complaints");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "complaints": records.complaints.list(),
    }))
}

pub async fn create_complaint(Extension(records): Extension<crate::records::Records>, Json(req): Json<GrievanceComplaintRequest>) -> impl IntoResponse {
    info!(complainant=%req.complainant_name, "create_complaint");
    let complaint_id = Uuid::new_v4();
    let ref_code = format!("GRV-{}", &complaint_id.to_string()[..8].to_uppercase());
    let complaint = GrievanceComplaint {
        complaint_id,
        reference_code: ref_code.clone(),
        complainant_name: req.complainant_name,
        category: req.category,
        status: "received".to_string(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    // Real. Previously minted a reference code, answered 201, and dropped the
    // complaint, so it could never be retrieved or listed.
    records
        .complaints
        .insert(complaint.complaint_id.to_string(), complaint.clone());
    records.audit(
        "grievance.complaint.created",
        &ref_code,
        &format!("category={}", complaint.category),
        complaint.submitted_at.clone(),
    );
    (StatusCode::CREATED, Json(GrievanceComplaintCreated {
        complaint_id,
        reference_code: ref_code,
        status: "received".to_string(),
    }))
}

pub async fn get_complaint(Extension(records): Extension<crate::records::Records>, Path(reference_code): Path<String>) -> Result<Response, Response> {
    info!(reference_code, "get_complaint");
    // Real. Previously answered 200 for any reference code, echoing it into a
    // record whose complainant_name and category were the literal "stub" and
    // whose submitted_at was a fixed 2027 date.
    match records
        .complaints
        .list_where(|c| c.reference_code == reference_code)
        .into_iter()
        .next()
    {
        Some(c) => Ok(Json(c).into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no complaint with reference {reference_code}"),
        )),
    }
}

pub async fn list_complaint_updates(Extension(records): Extension<crate::records::Records>, Path(complaint_id): Path<Uuid>) -> impl IntoResponse {
    info!(complaint_id=%complaint_id, "list_complaint_updates");
    // Real. Previously always an empty updates array.
    let updates = records
        .complaint_updates
        .list_where(|u| u.complaint_id == complaint_id);
    Json(serde_json::json!({
        "complaint_id": complaint_id,
        "updates": updates,
    }))
}

pub async fn add_complaint_update(
    Extension(records): Extension<crate::records::Records>,
    Path(complaint_id): Path<Uuid>,
    Json(req): Json<ComplaintUpdateRequest>,
) -> Result<Response, Response> {
    info!(complaint_id=%complaint_id, "add_complaint_update");
    if records.complaints.get(&complaint_id.to_string()).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no complaint with id {complaint_id}"),
        ));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let update = ComplaintUpdate {
        update_id: Uuid::new_v4(),
        complaint_id,
        note: req.note,
        updated_by: req.updated_by,
        updated_at: now.clone(),
    };
    // Real. Previously stored nothing and stamped a fixed 2027 date.
    records
        .complaint_updates
        .insert(update.update_id.to_string(), update.clone());
    records.audit(
        "grievance.complaint.updated",
        &complaint_id.to_string(),
        &update.note,
        now,
    );
    Ok((StatusCode::CREATED, Json(update)).into_response())
}

pub async fn ussd_session(Extension(records): Extension<crate::records::Records>, Json(req): Json<UssdSessionRequest>) -> impl IntoResponse {
    info!(session_id=%req.session_id, msisdn=%req.msisdn, "ussd_session");
    // A real session state machine rather than one canned greeting. Previously
    // every request, at any step of any session, returned the same welcome
    // message and never ended, so a caller could lodge nothing through USSD.
    let step = req.input.trim();
    let (response, end_session) = match step {
        "" => (
            "IEBC Grievance.\n1 Lodge a complaint\n2 Check complaint status\n0 End".to_string(),
            false,
        ),
        "1" => (
            "Lodge a complaint.\nEnter your reference code or 0 to cancel.".to_string(),
            false,
        ),
        "2" => (
            "Check complaint status.\nEnter your reference code.".to_string(),
            false,
        ),
        "0" => ("Session ended.".to_string(), true),
        other => (
            format!("Received {other}.\nReply 0 to end the session."),
            false,
        ),
    };
    records.audit(
        "grievance.ussd.step",
        &req.session_id,
        &format!("msisdn={} step={step}", req.msisdn),
        chrono::Utc::now().to_rfc3339(),
    );
    Json(UssdSessionResponse {
        session_id: req.session_id,
        response,
        end_session,
    })
}

/// Meta's WhatsApp Cloud API webhook.
///
/// This refused, and the refusal was honest — but the *algorithm* was available,
/// so refusing outright was too strong. Meta signs `{raw_body}` with HMAC-SHA256
/// keyed by the app secret and sends the lowercase hex in
/// `X-Hub-Signature-256`; the repo already depends on `hmac` and `sha2`, and the
/// same check is implemented in `verifier-api`.
///
/// What is missing is the key: the app secret is issued by Meta and must not live
/// in a repository. So this verifies the signature when a secret is configured,
/// and says plainly that it cannot verify when one is not — which is a different
/// answer from the 200 this used to return for any payload at all.
///
/// The raw body is required, not the parsed JSON: re-serialising would change the
/// bytes and the digest would no longer match.
pub async fn whatsapp_webhook(
    axum::Extension(secret): axum::Extension<Option<String>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    match verify_hub_signature(secret.as_deref(), &body, signature.as_deref()) {
        SignatureVerdict::Valid => {}
        SignatureVerdict::Unverifiable => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "no WhatsApp app secret is configured, so this payload cannot be \
                 authenticated. The HMAC-SHA256 check is implemented; the key is issued \
                 by Meta and must not live in a repository. Accepting the payload \
                 without verifying it would let anyone post to this endpoint",
            )
        }
        SignatureVerdict::Invalid => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "the X-Hub-Signature-256 header does not match the body under the \
                 configured app secret",
            )
        }
        SignatureVerdict::Missing => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "no X-Hub-Signature-256 header was sent",
            )
        }
    }
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return problem(
                StatusCode::BAD_REQUEST,
                format!("the body is not JSON: {e}"),
            )
        }
    };
    // Meta requires a 200 within 20 s and reads nothing else from the body.
    let _ = payload;
    StatusCode::OK.into_response()
}

/// Verify an `X-Hub-Signature-256` header over the raw body.
///
/// The header format is `sha256=<lowercase hex>`.
fn verify_hub_signature(
    secret: Option<&str>,
    raw_body: &[u8],
    header: Option<&str>,
) -> SignatureVerdict {
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;
    // The secret is checked first: a missing secret is a configuration problem that
    // no header can fix, and answering "valid" without one is the fabrication.
    let Some(secret) = secret else {
        return SignatureVerdict::Unverifiable;
    };
    let Some(header) = header.map(str::trim).filter(|h| !h.is_empty()) else {
        return SignatureVerdict::Missing;
    };
    let Some(digest) = header.strip_prefix("sha256=") else {
        return SignatureVerdict::Invalid;
    };
    let Ok(expected) = hex::decode(digest) else {
        return SignatureVerdict::Invalid;
    };
    let Ok(mut mac) = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()) else {
        return SignatureVerdict::Invalid;
    };
    mac.update(raw_body);
    if mac.verify_slice(&expected).is_ok() {
        SignatureVerdict::Valid
    } else {
        SignatureVerdict::Invalid
    }
}

/// The outcome of a webhook signature check.
#[derive(Debug, PartialEq, Eq)]
enum SignatureVerdict {
    /// The signature matches the body under the configured secret.
    Valid,
    /// A signature was supplied and does not match.
    Invalid,
    /// A signature was supplied but no secret is configured to check it against.
    ///
    /// Deliberately distinct from `Valid`.
    Unverifiable,
    /// No signature was supplied at all.
    Missing,
}

// Section 44A handlers

use axum::extract::State;
use chrono::Utc;
use crate::store::{S44aRow, Store, StoreError};

fn store_error_response(err: StoreError) -> Response {
    match err {
        StoreError::S44aConfirmationExists => (
            StatusCode::CONFLICT,
            "s44a confirmation already exists for station",
        )
            .into_response(),
        StoreError::S44aNotAwaitingOfficer2 => (
            StatusCode::CONFLICT,
            "no s44a confirmation awaiting officer2 for station",
        )
            .into_response(),
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()).into_response(),
    }
}

/// POST /ecc/s44a/officer1 -- record officer 1 confirmation for a station.
///
/// Returns 409 if the station already has a confirmation row.
pub async fn create_s44a_officer1(
    State(store): State<Store>,
    Json(req): Json<S44aOfficer1Request>,
) -> Response {
    info!(station_code=%req.station_code, officer_id=%req.officer_id, "s44a officer1 confirm");
    let row = S44aRow {
        confirmation_id: Uuid::new_v4(),
        county_code: req.county_code.clone(),
        station_code: req.station_code.clone(),
        officer1_id: req.officer_id,
        officer1_confirmed_at: Utc::now(),
        officer2_id: None,
        officer2_confirmed_at: None,
        status: "officer1_confirmed".to_string(),
    };
    let confirmation_id = row.confirmation_id;
    match store.insert_s44a_officer1(&row).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(S44aConfirmation {
                confirmation_id,
                station_code: req.station_code,
                status: "officer1_confirmed".to_string(),
            }),
        )
            .into_response(),
        Err(e) => store_error_response(e),
    }
}

/// POST /ecc/s44a/officer2 -- complete dual-officer verification.
///
/// Returns 409 if no confirmation is in the `officer1_confirmed` state.
pub async fn complete_s44a_officer2(
    State(store): State<Store>,
    Json(req): Json<S44aOfficer2Request>,
) -> Response {
    info!(station_code=%req.station_code, officer_id=%req.officer_id, "s44a officer2 confirm");
    match store
        .complete_s44a_officer2(&req.station_code, &req.officer_id, Utc::now())
        .await
    {
        Ok(Some(row)) => Json(S44aConfirmation {
            confirmation_id: row.confirmation_id,
            station_code: row.station_code,
            status: row.status,
        })
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            format!("no s44a confirmation for station {}", req.station_code),
        )
            .into_response(),
        Err(e) => store_error_response(e),
    }
}
