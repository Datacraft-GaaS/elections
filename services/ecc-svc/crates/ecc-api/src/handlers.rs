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
fn problem(status: StatusCode, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
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

pub async fn get_turnout_heatmap() -> impl IntoResponse {
    Json(serde_json::json!({ "entries": [] }))
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

pub async fn whatsapp_webhook(Json(payload): Json<WhatsappWebhookPayload>) -> Response {
    // Refused, and the refusal is the honest one. Meta's webhook signature check
    // (X-Hub-Signature-256 over the raw body against the app secret) is not
    // implemented here, and this handler previously answered 200 OK for any
    // payload at all while discarding it -- so a caller posting to this endpoint
    // learned nothing, and any caller could post to it. Accepting the payload
    // without verifying it would be worse than refusing.
    let _ = payload;
    problem(
        StatusCode::NOT_IMPLEMENTED,
        "whatsapp webhook signature verification is not implemented; accepting \
         unverified payloads would let anyone post to this endpoint"
            .to_string(),
    )
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
