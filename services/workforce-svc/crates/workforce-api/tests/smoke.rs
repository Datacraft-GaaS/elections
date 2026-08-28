use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const HASH_A: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const HASH_B: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const HASH_C: &str = "3333333333333333333333333333333333333333333333333333333333333333";

fn mem_app() -> axum::Router {
    workforce_api::router::build_router(workforce_api::store::Store::Mem(Default::default()))
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let req = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            builder.body(Body::from(b.to_string())).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn health_returns_200() {
    let (status, _) = send(&mem_app(), "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

/// Full happy path: create case → screen → documents_complete(gate ok) →
/// evaluate(gate passed) → select → appoint(contract ref) → worker exists.
#[tokio::test]
async fn recruitment_happy_path_appoints_worker_with_eligibility() {
    let app = mem_app();

    // Create the recruitment case at `applied`.
    let (status, case) = send(
        &app,
        "POST",
        "/v1/workforce/recruitment",
        Some(json!({
            "application_ref": "M55-000123",
            "applicant_name": "Ada Oburu",
            "national_id_hash": HASH_A,
            "role_code": "PO",
            "county": "KE-030",
            "composite_score": 88.5
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(case["stage"]["state"], "applied");
    let id = case["case_id"].as_str().unwrap().to_string();

    // Illegal jump straight to appointed => 409 with allowed list.
    let (status, err) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "seal"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let allowed = err["allowed"].as_array().unwrap();
    assert!(allowed.contains(&json!("screened")));
    assert!(allowed.contains(&json!("rejected")));

    // Advance through screening.
    let (status, case) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "screened", "actor": "hr-clerk-1"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case["stage"]["state"], "screened");

    // documents_complete without the gate payload => 422.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "documents_complete", "actor": "compliance-1"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // With the support-admin completeness probe the transition succeeds.
    let (status, case) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({
            "to": "documents_complete",
            "actor": "compliance-1",
            "documents_probe": {"complete": true}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case["stage"]["state"], "documents_complete");

    // evaluated without exam result => 422; with a passing result => ok.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "evaluated", "actor": "exam-board"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, case) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({
            "to": "evaluated",
            "actor": "exam-board",
            "exam_result": {"passed": true, "session_id": "EXAM-77"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case["stage"]["state"], "evaluated");

    let (status, case) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "selected", "actor": "cro-030"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case["stage"]["state"], "selected");

    // Appointment requires contract_ref.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "commission-seal"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, case) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "commission-seal", "contract_ref": "SC-2027-000451"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case["stage"]["state"], "appointed");
    let history = case["stage_history"].as_array().unwrap();
    assert_eq!(history.len(), 5);
    assert_eq!(history[0]["from"], "applied");

    // The worker exists with role eligibility derived from the case.
    let (status, listing) = send(
        &app,
        "GET",
        "/v1/workforce/workers?home_county=KE-030&role=PO",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let workers = listing["workers"].as_array().unwrap();
    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0]["legal_name"], "Ada Oburu");
    assert_eq!(workers[0]["national_id_hash"], HASH_A);
    assert_eq!(workers[0]["role_eligibility"], json!(["PO"]));
    assert_eq!(workers[0]["payroll_contract_ref"], "SC-2027-000451");
}

/// Appointing the same case twice is rejected as an illegal transition.
#[tokio::test]
async fn double_appointment_returns_409() {
    let app = mem_app();
    let (_, case) = send(
        &app,
        "POST",
        "/v1/workforce/recruitment",
        Some(json!({
            "application_ref": "M55-000777",
            "applicant_name": "Baraka Mwaura",
            "national_id_hash": HASH_B,
            "role_code": "DPO",
            "county": "KE-042"
        })),
    )
    .await;
    let id = case["case_id"].as_str().unwrap().to_string();
    for to in ["screened", "documents_complete", "evaluated", "selected"] {
        let extra = match to {
            "documents_complete" => {
                json!({"to": to, "actor": "a", "documents_probe": {"complete": true}})
            }
            "evaluated" => {
                json!({"to": to, "actor": "a", "exam_result": {"passed": true, "session_id": "S"}})
            }
            _ => json!({"to": to, "actor": "a"}),
        };
        let (status, _) = send(
            &app,
            "POST",
            format!("/v1/workforce/recruitment/{id}/transition").as_str(),
            Some(extra),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "advance to {to}");
    }
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "seal", "contract_ref": "SC-1"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Second appointment of the same case conflicts.
    let (status, err) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "seal", "contract_ref": "SC-2"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(err["allowed"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn contact_upsert_preferred_rule_verify_and_history() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Cynthia Wanjiru",
            "national_id_hash": HASH_C,
            "home_county": "KE-011",
            "role_eligibility": ["CLERK"],
            "next_of_kin": {"name": "Grace Wanjiru", "phone_hash": HASH_B}
        })),
    )
    .await;
    assert_eq!(worker["next_of_kin_name"], "Grace Wanjiru");
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    // Add first phone contact, preferred + consented.
    let (status, c1) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_A, "preferred": true, "consented": true, "actor": "clerk-9"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(c1["preferred"].as_bool().unwrap());

    // Add a second phone number marked preferred: first one must be demoted.
    let (status, c2) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_B, "preferred": true, "consented": true, "actor": "clerk-9"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(c2["preferred"].as_bool().unwrap());

    let (_, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/contacts").as_str(),
        None,
    )
    .await;
    let contacts = listing["contacts"].as_array().unwrap();
    assert_eq!(contacts.len(), 2);
    let preferred: Vec<&Value> = contacts
        .iter()
        .filter(|c| c["preferred"].as_bool().unwrap())
        .collect();
    assert_eq!(preferred.len(), 1, "at most one preferred per channel");
    assert_eq!(preferred[0]["value_hash_hex"], HASH_B);

    // Invalid hash => 422.
    let (status, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/sms").as_str(),
        Some(json!({"value_hash_hex": "nothex"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Verify marks the preferred phone verified and logs it.
    let (status, verified) = send(
        &app,
        "POST",
        format!("/v1/workforce/workers/{wid}/contacts/phone/verify").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(verified["value_hash_hex"], HASH_B);
    assert!(verified["verified"].as_bool().unwrap());

    // Change history is append-only and ordered.
    let (_, history) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/contact-history").as_str(),
        None,
    )
    .await;
    let entries = history["entries"].as_array().unwrap();
    let actions: Vec<&str> = entries
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        vec![
            "added",
            "consent_granted",
            "added",
            "consent_granted",
            "verified"
        ]
    );

    // Removal requires a reason => 422 without one.
    let (status, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_A, "remove": true})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // With a reason the removal succeeds and is logged.
    let (status, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_A, "remove": true, "reason": "number recycled", "actor": "clerk-9"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/contacts").as_str(),
        None,
    )
    .await;
    let contacts = listing["contacts"].as_array().unwrap();
    assert_eq!(contacts.len(), 1);
    let (_, history) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/contact-history").as_str(),
        None,
    )
    .await;
    let actions: Vec<&str> = history["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions.last().copied(), Some("removed"));
}

#[tokio::test]
async fn posting_supersession_chain_and_equipment_lifecycle() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Daniel Kiptoo",
            "national_id_hash": HASH_A,
            "home_county": "KE-036",
            "role_eligibility": ["PO"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    // First posting.
    let (status, p1) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-036-001",
            "role_code": "PO",
            "effective_from": "2027-08-07T00:00:00Z",
            "posted_by": "cro-036"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let p1_id = p1["posting_id"].as_str().unwrap().to_string();

    // Second posting for the same worker supersedes the first automatically.
    let (status, p2) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-036-002",
            "role_code": "PO",
            "effective_from": "2027-08-08T00:00:00Z",
            "posted_by": "cro-036"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let p2_id = p2["posting_id"].as_str().unwrap().to_string();
    assert_ne!(p1_id, p2_id);

    let (_, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/postings?worker_id={wid}").as_str(),
        None,
    )
    .await;
    let postings = listing["postings"].as_array().unwrap();
    assert_eq!(postings.len(), 2);
    let old = postings
        .iter()
        .find(|p| p["posting_id"] == json!(p1_id))
        .unwrap();
    assert_eq!(old["status"], "superseded");
    assert_eq!(old["superseded_by"], json!(p2_id));
    let newp = postings
        .iter()
        .find(|p| p["posting_id"] == json!(p2_id))
        .unwrap();
    assert_eq!(newp["status"], "active");

    // Superseding an already-superseded posting conflicts.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p1_id}/supersede").as_str(),
        Some(json!({
            "new_station_code": "STN-036-009",
            "effective_from": "2027-08-09T00:00:00Z",
            "posted_by": "cro-036"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Superseding the active posting creates a successor in the chain.
    let (status, p3) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p2_id}/supersede").as_str(),
        Some(json!({
            "new_station_code": "STN-036-003",
            "effective_from": "2027-08-09T00:00:00Z",
            "posted_by": "cro-036"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(p3["status"], "active");

    // Equipment claim against the superseded posting conflicts.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p2_id}/equipment").as_str(),
        Some(json!({"kit_serials": ["BVI-0001"]})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Claim against the active posting succeeds; a second open claim conflicts.
    let p3_id = p3["posting_id"].as_str().unwrap().to_string();
    let (status, claim) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p3_id}/equipment").as_str(),
        Some(json!({"kit_serials": ["BVI-0001", "PHN-0001", "SEAL-001"]})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let claim_id = claim["claim_id"].as_str().unwrap().to_string();

    let (status, _err) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p3_id}/equipment").as_str(),
        Some(json!({"kit_serials": ["EXTRA-1"]})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "claim while previous unreturned"
    );

    let (status, claim) = send(
        &app,
        "POST",
        format!("/v1/workforce/equipment/{claim_id}/acknowledge").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(claim["acknowledged"].as_bool().unwrap());
    assert!(!claim["returned"].as_bool().unwrap());

    // Flag a lost seal before returning.
    let (status, claim) = send(
        &app,
        "POST",
        format!("/v1/workforce/equipment/{claim_id}/flag-loss").as_str(),
        Some(json!({"serials": ["SEAL-001"], "reason": "not found at handover"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        claim["loss_flags"],
        json!(["SEAL-001:not found at handover"])
    );

    let (status, claim) = send(
        &app,
        "POST",
        format!("/v1/workforce/equipment/{claim_id}/return").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(claim["returned"].as_bool().unwrap());

    // After return, a fresh claim is allowed.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{p3_id}/equipment").as_str(),
        Some(json!({"kit_serials": ["REPL-1"]})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn task_lifecycle_critical_evidence_and_idempotent_ack() {
    let app = mem_app();

    // Worker + posting + templates.
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Esther Njoki",
            "national_id_hash": HASH_B,
            "home_county": "KE-022",
            "role_eligibility": ["CLERK"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();
    let (_, posting) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-022-004",
            "role_code": "CLERK",
            "effective_from": "2027-08-07T00:00:00Z",
            "posted_by": "cro-022"
        })),
    )
    .await;
    let pid = posting["posting_id"].as_str().unwrap().to_string();

    let (status, critical_tmpl) = send(
        &app,
        "POST",
        "/v1/workforce/task-templates",
        Some(json!({"role_code": "CLERK", "phase": "pre_poll", "title": "Verify ballot box seals", "critical": true})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let ct_id = critical_tmpl["template_id"].as_str().unwrap().to_string();

    let (status, plain_tmpl) = send(
        &app,
        "POST",
        "/v1/workforce/task-templates",
        Some(
            json!({"role_code": "CLERK", "phase": "polling_day", "title": "Hourly turnout report"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let pt_id = plain_tmpl["template_id"].as_str().unwrap().to_string();

    // Template filters.
    let (_, list) = send(
        &app,
        "GET",
        "/v1/workforce/task-templates?role=CLERK&phase=pre_poll",
        None,
    )
    .await;
    assert_eq!(list["templates"].as_array().unwrap().len(), 1);

    let (_, assignment) = send(
        &app,
        "POST",
        "/v1/workforce/tasks",
        Some(json!({
            "posting_id": pid,
            "template_id": ct_id,
            "due_window": {"from": "2027-08-08T00:00:00Z", "to": "2027-08-08T18:00:00Z"}
        })),
    )
    .await;
    let aid = assignment["assignment_id"].as_str().unwrap().to_string();
    assert_eq!(assignment["status"], "pending");

    // Critical done before ack => conflict (chain pending -> acked -> done).
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{aid}/done").as_str(),
        Some(json!({"evidence_hash": HASH_C})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Ack is idempotent: repeat calls return 200.
    for _ in 0..2 {
        let (status, a) = send(
            &app,
            "POST",
            format!("/v1/workforce/tasks/{aid}/ack").as_str(),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(a["status"], "acked");
    }

    // Critical done without evidence => 422; with evidence => done.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{aid}/done").as_str(),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{aid}/done").as_str(),
        Some(json!({"evidence_hash": "zzzz"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, a) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{aid}/done").as_str(),
        Some(json!({"evidence_hash": HASH_C})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(a["status"], "done");
    assert_eq!(a["evidence_hash"], HASH_C);

    // Repeated identical done call is idempotent.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{aid}/done").as_str(),
        Some(json!({"evidence_hash": HASH_C})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Non-critical task can complete without evidence but can block instead.
    let (_, b_assignment) = send(
        &app,
        "POST",
        "/v1/workforce/tasks",
        Some(json!({
            "posting_id": pid,
            "template_id": pt_id,
            "due_window": {"from": "2027-08-09T04:00:00Z", "to": "2027-08-09T18:00:00Z"}
        })),
    )
    .await;
    let bid = b_assignment["assignment_id"].as_str().unwrap().to_string();
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/tasks/{bid}/blocked").as_str(),
        Some(json!({"reason": "BVI offline"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, list) = send(
        &app,
        "GET",
        format!("/v1/workforce/tasks?posting_id={pid}&status=blocked").as_str(),
        None,
    )
    .await;
    let blocked = list["assignments"].as_array().unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0]["block_reason"], "BVI offline");
}

#[tokio::test]
async fn performance_append_only_records_with_summary() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Fred Ochieng",
            "national_id_hash": HASH_C,
            "home_county": "KE-045",
            "role_eligibility": ["PO"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    for record in [
        json!({"category": "attendance", "score": 4.5, "detail": "checked in early", "recorded_by": "ro-045"}),
        json!({"category": "punctuality", "score": 4.0, "detail": "on-time open", "recorded_by": "ro-045"}),
        json!({"category": "incident", "detail": "queue dispute escalated late", "recorded_by": "ecc"}),
        json!({"category": "evaluation", "score": 3.5, "detail": "end-of-election rubric", "recorded_by": "ro-045"}),
    ] {
        let mut body = record;
        body["worker_id"] = json!(wid);
        let (status, _) = send(&app, "POST", "/v1/workforce/performance", Some(body)).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, resp) = send(
        &app,
        "GET",
        format!("/v1/workforce/performance?worker_id={wid}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["records"].as_array().unwrap().len(), 4);
    assert_eq!(resp["summary"]["incident_count"], 1);
    let avg = resp["summary"]["avg_score"].as_f64().unwrap();
    assert!(
        (avg - 4.0).abs() < 1e-6,
        "avg of 4.5, 4.0, 3.5 == 4.0, got {avg}"
    );

    // Missing worker_id param => 422.
    let (status, _) = send(&app, "GET", "/v1/workforce/performance", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn worker_profile_update_and_payment_notification_resolve_preferred() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Grace Moraa",
            "national_id_hash": HASH_A,
            "home_county": "KE-012",
            "role_eligibility": ["CLERK"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    // Update bank/payroll ref and next of kin without touching other fields.
    let (status, updated) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}").as_str(),
        Some(json!({
            "payroll_contract_ref": "SC-9001",
            "next_of_kin": {"name": "Peter Moraa", "phone_hash": HASH_B}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["payroll_contract_ref"], "SC-9001");
    assert_eq!(updated["next_of_kin_name"], "Peter Moraa");
    assert_eq!(updated["home_county"], "KE-012");

    // Invalid next-of-kin hash is rejected.
    let (status, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}").as_str(),
        Some(json!({"next_of_kin": {"name": "X", "phone_hash": "bad"}})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Register a preferred phone contact point.
    let (status, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_C, "preferred": true, "consented": true})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // A payment notification (from payroll) without an explicit channel
    // resolves to the preferred contact point, carrying only the hash.
    let (status, n) = send(
        &app,
        "POST",
        "/v1/workforce/notifications",
        Some(json!({
            "worker_id": wid,
            "kind": "payment",
            "subject": "Payment",
            "body": "KES 12,000 disbursed"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(n["channel"], "phone");
    assert_eq!(n["delivery_ref"], HASH_C);

    let (status, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/notifications?worker_id={wid}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listing["notifications"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn interview_records_are_created_and_listed() {
    let app = mem_app();
    let (_, case) = send(
        &app,
        "POST",
        "/v1/workforce/recruitment",
        Some(json!({
            "application_ref": "M55-INT-01",
            "applicant_name": "Naomi Achieng",
            "national_id_hash": HASH_B,
            "role_code": "DPO",
            "county": "KE-020"
        })),
    )
    .await;
    let id = case["case_id"].as_str().unwrap().to_string();

    // Missing interviewer => 422.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/interviews").as_str(),
        Some(json!({"outcome": "recommend"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, first) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/interviews").as_str(),
        Some(json!({
            "interviewer": "panel-1",
            "score": 3.5,
            "outcome": "recommend",
            "notes": "strong procedural knowledge"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(first["outcome"], "recommend");

    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/interviews").as_str(),
        Some(json!({"interviewer": "panel-1", "outcome": "hold"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/recruitment/{id}/interviews").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let interviews = listing["interviews"].as_array().unwrap();
    assert_eq!(interviews.len(), 2);
    assert_eq!(interviews[0]["outcome"], "recommend");
}

#[tokio::test]
async fn posting_and_training_fire_notifications() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Wanjiku Njeri",
            "national_id_hash": HASH_C,
            "home_county": "KE-029",
            "role_eligibility": ["PO"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    // Preferred contact so notifications carry a delivery target.
    let (_, _) = send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/sms").as_str(),
        Some(json!({"value_hash_hex": HASH_A, "preferred": true, "consented": true})),
    )
    .await;

    let (_, posting) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-029-001",
            "role_code": "PO",
            "effective_from": "2027-08-07T00:00:00Z",
            "posted_by": "cro-029"
        })),
    )
    .await;
    let pid = posting["posting_id"].as_str().unwrap().to_string();

    // Posting triggers a posting notification automatically.
    let (status, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/notifications?worker_id={wid}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let notifs = listing["notifications"].as_array().unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0]["kind"], "posting");
    assert_eq!(notifs[0]["channel"], "sms");
    assert_eq!(notifs[0]["delivery_ref"], HASH_A);

    // A template linked to just-in-time training fires a training notification.
    let (_, tmpl) = send(
        &app,
        "POST",
        "/v1/workforce/task-templates",
        Some(json!({
            "role_code": "PO",
            "phase": "pre_poll",
            "title": "Zero the BVI kit",
            "critical": true,
            "training_content_id": "TRN-BVI-ZERO-01"
        })),
    )
    .await;
    let tid = tmpl["template_id"].as_str().unwrap().to_string();
    assert_eq!(tmpl["training_content_id"], "TRN-BVI-ZERO-01");

    let (status, _) = send(
        &app,
        "POST",
        "/v1/workforce/tasks",
        Some(json!({
            "posting_id": pid,
            "template_id": tid,
            "due_window": {"from": "2027-08-08T00:00:00Z", "to": "2027-08-08T18:00:00Z"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, listing) = send(
        &app,
        "GET",
        format!("/v1/workforce/notifications?worker_id={wid}").as_str(),
        None,
    )
    .await;
    let notifs = listing["notifications"].as_array().unwrap();
    assert_eq!(notifs.len(), 2);
    assert_eq!(notifs[1]["kind"], "training");
    assert!(notifs[1]["body"]
        .as_str()
        .unwrap()
        .contains("TRN-BVI-ZERO-01"));
}

/// Domain operations must emit their design §8 cross-service events. The Mem
/// store records envelopes so the contract is assertable hermetically.
#[tokio::test]
async fn domain_operations_emit_cross_service_events() {
    let store = workforce_api::store::Store::Mem(Default::default());
    let app = workforce_api::router::build_router(store.clone());

    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Emit Test",
            "national_id_hash": HASH_A,
            "home_county": "KE-010",
            "role_eligibility": ["PO"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    // Posting -> posting.activated.
    let (status, posting) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-010-001",
            "role_code": "PO",
            "effective_from": "2027-08-07T00:00:00Z",
            "posted_by": "cro-010"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let pid = posting["posting_id"].as_str().unwrap().to_string();

    // Performance -> performance.evaluation_recorded.
    let (status, _) = send(
        &app,
        "POST",
        "/v1/workforce/performance",
        Some(json!({
            "worker_id": wid,
            "category": "attendance",
            "score": 4.0,
            "detail": "early",
            "recorded_by": "ro-010"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Equipment issue -> kit_issued; return -> kit_returned.
    let (_, claim) = send(
        &app,
        "POST",
        format!("/v1/workforce/postings/{pid}/equipment").as_str(),
        Some(json!({"kit_serials": ["BVI-0001"]})),
    )
    .await;
    let cid = claim["claim_id"].as_str().unwrap().to_string();
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/equipment/{cid}/return").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let topics: Vec<String> = store
        .emitted_events()
        .await
        .iter()
        .map(|e| e.topic.clone())
        .collect();
    for expected in [
        "eel.workforce.posting.activated.v1",
        "eel.workforce.performance.evaluation_recorded.v1",
        "eel.workforce.equipment.kit_issued.v1",
        "eel.workforce.equipment.kit_returned.v1",
    ] {
        assert!(
            topics.iter().any(|t| t.as_str() == expected),
            "missing {expected}; got {topics:?}"
        );
    }
}

/// Appointment emits both the stage-change and worker-appointed events.
#[tokio::test]
async fn worker_badge_returns_verifiable_credential() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({
            "legal_name": "Grace Moraa",
            "national_id_hash": HASH_A,
            "home_county": "KE-030",
            "role_eligibility": ["PO"]
        })),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();

    let (status, _posting) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({
            "worker_id": wid,
            "station_code": "STN-030-001",
            "role_code": "PO",
            "effective_from": "2027-08-07T00:00:00Z",
            "posted_by": "cro-030"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, badge) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/badge").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(badge["worker_id"], json!(wid));
    assert_eq!(badge["name"], "Grace Moraa");
    assert_eq!(badge["role"], "PO");
    assert_eq!(badge["station_code"], "STN-030-001");
    let hash = badge["badge_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64, "badge_hash must be SHA-256 hex");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // The hash is deterministic (excludes issued_at), so a second fetch agrees.
    let (_, again) = send(
        &app,
        "GET",
        format!("/v1/workforce/workers/{wid}/badge").as_str(),
        None,
    )
    .await;
    assert_eq!(again["badge_hash"], badge["badge_hash"]);

    // Unknown worker -> 404.
    let (status, _) = send(
        &app,
        "GET",
        "/v1/workforce/workers/00000000-0000-4000-8000-000000000000/badge",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn readiness_reports_staffing_coverage_and_shortfalls() {
    let app = mem_app();
    let station = "STN-010-001";

    // One PO, one DPO, one clerk — clerk short by one against the 1/1/2 floor.
    for (name, role) in [("Po One", "PO"), ("Dpo One", "DPO"), ("Clerk One", "CLERK")] {
        let (_, worker) = send(
            &app,
            "POST",
            "/v1/workforce/workers",
            Some(json!({
                "legal_name": name,
                "national_id_hash": HASH_B,
                "home_county": "KE-010",
                "role_eligibility": [role]
            })),
        )
        .await;
        let wid = worker["worker_id"].as_str().unwrap().to_string();
        let (status, _) = send(
            &app,
            "POST",
            "/v1/workforce/postings",
            Some(json!({
                "worker_id": wid,
                "station_code": station,
                "role_code": role,
                "effective_from": "2027-08-07T00:00:00Z",
                "posted_by": "cro-010"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, resp) = send(
        &app,
        "GET",
        format!("/v1/workforce/readiness?station={station}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let station = &resp["stations"][0];
    assert_eq!(station["station_code"], json!("STN-010-001"));
    assert_eq!(station["staffing"], json!({"PO": 1, "DPO": 1, "CLERK": 1}));
    assert_eq!(station["staffing_shortfalls"], json!(["CLERK:1/2"]));
    // Filled 3 of 4 required units -> 75%.
    assert!((station["coverage_pct"].as_f64().unwrap() - 75.0).abs() < 1e-6);
}

#[tokio::test]
async fn bulk_posting_creates_and_collects_errors() {
    let app = mem_app();
    let mut ids = Vec::new();
    for (name, hash) in [("A", HASH_A), ("B", HASH_B)] {
        let (_, w) = send(
            &app,
            "POST",
            "/v1/workforce/workers",
            Some(json!({"legal_name": name, "national_id_hash": hash, "home_county": "KE-030", "role_eligibility": ["PO"]})),
        )
        .await;
        ids.push(w["worker_id"].as_str().unwrap().to_string());
    }

    let (status, resp) = send(
        &app,
        "POST",
        "/v1/workforce/postings/bulk",
        Some(json!({"postings": [
            {"worker_id": ids[0], "station_code": "STN-030-001", "role_code": "PO", "effective_from": "2027-08-07T00:00:00Z", "posted_by": "cro"},
            {"worker_id": ids[1], "station_code": "STN-030-002", "role_code": "PO", "effective_from": "2027-08-07T00:00:00Z", "posted_by": "cro"},
            {"worker_id": "00000000-0000-4000-8000-000000000000", "station_code": "STN-030-003", "role_code": "PO", "effective_from": "2027-08-07T00:00:00Z", "posted_by": "cro"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["created"].as_array().unwrap().len(), 2);
    assert_eq!(resp["errors"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn readiness_filters_by_county_and_rolls_up_counties() {
    let app = mem_app();
    for (name, county) in [("A", "KE-030"), ("B", "KE-042")] {
        let (_, w) = send(
            &app,
            "POST",
            "/v1/workforce/workers",
            Some(json!({"legal_name": name, "national_id_hash": HASH_A, "home_county": county, "role_eligibility": ["PO"]})),
        )
        .await;
        let wid = w["worker_id"].as_str().unwrap().to_string();
        send(
            &app,
            "POST",
            "/v1/workforce/postings",
            Some(json!({"worker_id": wid, "station_code": format!("STN-{}-001", &county[3..]), "role_code": "PO", "effective_from": "2027-08-07T00:00:00Z", "posted_by": "cro"})),
        )
        .await;
    }

    let (status, all) = send(&app, "GET", "/v1/workforce/readiness", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all["counties"].as_array().unwrap().len(), 2);

    let (status, scoped) = send(&app, "GET", "/v1/workforce/readiness?county=KE-030", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(scoped["stations"].as_array().unwrap().len(), 1);
    assert_eq!(scoped["counties"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn audit_trail_returns_stage_and_contact_changes() {
    let app = mem_app();
    let (_, case) = send(
        &app,
        "POST",
        "/v1/workforce/recruitment",
        Some(json!({"application_ref": "M55-AUD", "applicant_name": "Audit Person", "national_id_hash": HASH_C, "role_code": "PO", "county": "KE-030"})),
    )
    .await;
    let id = case["case_id"].as_str().unwrap().to_string();
    send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "screened", "actor": "hr-1"})),
    )
    .await;

    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({"legal_name": "Audit Worker", "national_id_hash": HASH_A, "home_county": "KE-030"})),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();
    send(
        &app,
        "PUT",
        format!("/v1/workforce/workers/{wid}/contacts/phone").as_str(),
        Some(json!({"value_hash_hex": HASH_B, "actor": "clerk-1"})),
    )
    .await;

    let (status, audit) = send(&app, "GET", "/v1/workforce/audit", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(audit["stage_changes"].as_array().unwrap().len(), 1);
    assert_eq!(audit["stage_changes"][0]["to"], "screened");
    assert_eq!(audit["contact_changes"].as_array().unwrap().len(), 1);
    assert_eq!(audit["contact_changes"][0]["actor"], "clerk-1");
}

#[tokio::test]
async fn task_listing_includes_training_deep_link() {
    let app = mem_app();
    let (_, worker) = send(
        &app,
        "POST",
        "/v1/workforce/workers",
        Some(json!({"legal_name": "Jit Person", "national_id_hash": HASH_A, "home_county": "KE-030", "role_eligibility": ["PO"]})),
    )
    .await;
    let wid = worker["worker_id"].as_str().unwrap().to_string();
    let (_, posting) = send(
        &app,
        "POST",
        "/v1/workforce/postings",
        Some(json!({"worker_id": wid, "station_code": "STN-030-001", "role_code": "PO", "effective_from": "2027-08-07T00:00:00Z", "posted_by": "cro"})),
    )
    .await;
    let pid = posting["posting_id"].as_str().unwrap().to_string();
    let (_, tmpl) = send(
        &app,
        "POST",
        "/v1/workforce/task-templates",
        Some(json!({"role_code": "PO", "phase": "pre_poll", "title": "Zero the BVI", "critical": true, "training_content_id": "TRN-BVI-ZERO"})),
    )
    .await;
    let tid = tmpl["template_id"].as_str().unwrap().to_string();
    send(
        &app,
        "POST",
        "/v1/workforce/tasks",
        Some(json!({"posting_id": pid, "template_id": tid, "due_window": {"from": "2027-08-08T00:00:00Z", "to": "2027-08-08T18:00:00Z"}})),
    )
    .await;

    let (status, list) = send(
        &app,
        "GET",
        format!("/v1/workforce/tasks?posting_id={pid}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let a = &list["assignments"][0];
    assert_eq!(a["title"], "Zero the BVI");
    assert_eq!(a["critical"], true);
    assert_eq!(a["training_content_id"], "TRN-BVI-ZERO");
}

#[tokio::test]
async fn appointment_emits_stage_changed_and_worker_appointed() {
    let store = workforce_api::store::Store::Mem(Default::default());
    let app = workforce_api::router::build_router(store.clone());

    let (_, case) = send(
        &app,
        "POST",
        "/v1/workforce/recruitment",
        Some(json!({
            "application_ref": "M55-EVT-01",
            "applicant_name": "Event Emitter",
            "national_id_hash": HASH_C,
            "role_code": "PO",
            "county": "KE-030"
        })),
    )
    .await;
    let id = case["case_id"].as_str().unwrap().to_string();

    for to in ["screened", "documents_complete", "evaluated", "selected"] {
        let extra = match to {
            "documents_complete" => {
                json!({"to": to, "actor": "a", "documents_probe": {"complete": true}})
            }
            "evaluated" => {
                json!({"to": to, "actor": "a", "exam_result": {"passed": true, "session_id": "S"}})
            }
            _ => json!({"to": to, "actor": "a"}),
        };
        let (status, _) = send(
            &app,
            "POST",
            format!("/v1/workforce/recruitment/{id}/transition").as_str(),
            Some(extra),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "advance to {to}");
    }
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/workforce/recruitment/{id}/transition").as_str(),
        Some(json!({"to": "appointed", "actor": "seal", "contract_ref": "SC-1"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let events = store.emitted_events().await;
    let topics: Vec<String> = events.iter().map(|e| e.topic.clone()).collect();
    // 5 transitions -> 5 stage_changed events, plus 1 worker.appointed.
    assert_eq!(
        topics
            .iter()
            .filter(|t| t.as_str() == "eel.workforce.application.stage_changed.v1")
            .count(),
        5
    );
    assert!(
        topics
            .iter()
            .any(|t| t.as_str() == "eel.workforce.worker.appointed.v1"),
        "got {topics:?}"
    );

    // The appointed event carries the worker id and hash, never the legal name.
    let appointed = events
        .iter()
        .find(|e| e.topic == "eel.workforce.worker.appointed.v1")
        .unwrap();
    assert!(appointed.data.get("worker_id").is_some());
    assert!(appointed.data.get("legal_name").is_none());
}

#[test]
fn gate_check_respects_config() {
    use workforce_api::domain::{GateConfig, TransitionRequest};
    use workforce_api::handlers::gate_check;

    let req = TransitionRequest {
        to: "evaluated".into(),
        actor: "a".into(),
        reason: None,
        documents_probe: None,
        exam_result: None,
        contract_ref: None,
    };
    // Exam gate on by default -> fails without an exam result.
    assert!(gate_check("evaluated", &GateConfig::all(), &req).is_some());
    // Exam gate disabled -> the transition proceeds without an exam result.
    let no_exam = GateConfig {
        documents: true,
        exam: false,
        contract: true,
    };
    assert!(gate_check("evaluated", &no_exam, &req).is_none());
}

#[test]
fn gate_config_parses_subset() {
    use workforce_api::domain::GateConfig;
    let g = GateConfig::parse("documents,contract");
    assert!(g.documents && !g.exam && g.contract);
    assert!(!GateConfig::parse("").documents);
    assert_eq!(
        GateConfig::parse("documents,exam,contract"),
        GateConfig::all()
    );
}
