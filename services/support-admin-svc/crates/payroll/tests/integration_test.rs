//! Integration tests for the payroll crate (WP-5.6).

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use chrono::{Duration, Utc};
use payroll::domain::{
    ApplicationStatus, Decision, DeductionType, DocumentCompleteness, DocumentRequirement,
    DocumentSubmission, DomainError, JobApplication, PayrollRun, PayrollStatus, ShortlistDecision,
    Verdict,
};
use payroll::handlers;
use payroll::state::AppState;
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

fn make_application(training_passed: bool) -> JobApplication {
    JobApplication {
        id: Uuid::now_v7(),
        application_ref: "PWP-TEST-0001".into(),
        role_code: "clerk".into(),
        county: "KE-022".into(),
        applicant_name: "Test Applicant".into(),
        national_id_hash: "deadbeef".into(),
        contact_phone_hash: None,
        contact_email_hash: None,
        status: ApplicationStatus::Applied,
        applied_at: Utc::now(),
        training_gate_passed: training_passed,
    }
}

#[test]
fn shortlist_requires_training_gate() {
    let app = make_application(false);
    assert!(matches!(
        ShortlistDecision::validate_shortlist(&app),
        Err(DomainError::TrainingGateNotPassed)
    ));
}

#[test]
fn shortlist_passes_when_trained() {
    let app = make_application(true);
    ShortlistDecision::validate_shortlist(&app).unwrap();
}

#[test]
fn payroll_sod_enforced() {
    let uid = Uuid::now_v7();
    let run = PayrollRun {
        id: Uuid::now_v7(),
        election_id: Uuid::now_v7(),
        period: "2027-08".into(),
        status: PayrollStatus::Draft,
        total_staff: 0,
        total_gross_kes: 0,
        initiated_by: uid,
        approved_by: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    assert!(matches!(
        run.validate_approval(uid),
        Err(DomainError::SodViolation)
    ));
    assert!(run.validate_approval(Uuid::now_v7()).is_ok());
}

#[test]
fn decision_serialises() {
    let d = Decision::Shortlist;
    let s = serde_json::to_string(&d).unwrap();
    assert_eq!(s, r#""shortlist""#);
}

#[test]
fn deduction_type_serialises() {
    let d = DeductionType::Paye;
    let s = serde_json::to_string(&d).unwrap();
    assert_eq!(s, r#""paye""#);
}

// ---------------------------------------------------------------------------
// Documents: submission & verification (workforce-lifecycle §2)
// ---------------------------------------------------------------------------

mod documents {
    use super::*;
    use payroll::handlers::{
        BulkVerdictItem, BulkVerdictRequest, CompletenessQuery, ExpirySweepQuery,
        ListRequirementsQuery, SubmitDocumentRequest, SupersedeDocumentRequest, VerdictRequest,
    };

    fn sha256_hex(seed: &str) -> String {
        hex::encode(Sha256::digest(seed.as_bytes()))
    }

    fn submit_req(app: Uuid, doc_type: &str, seed: &str) -> SubmitDocumentRequest {
        SubmitDocumentRequest {
            application_id: app,
            doc_type: doc_type.into(),
            content_hash_hex: sha256_hex(seed),
            object_key: format!("tenants/{app}/{doc_type}/{seed}.pdf"),
            channel: "web".into(),
        }
    }

    #[tokio::test]
    async fn requirements_seeded_per_role() {
        let po = handlers::list_document_requirements(Query(ListRequirementsQuery {
            role_code: Some("po".into()),
        }))
        .await
        .0
        .get("items")
        .and_then(|v| v.as_array())
        .unwrap()
        .clone();
        assert_eq!(po.len(), 7);

        let clerk = handlers::list_document_requirements(Query(ListRequirementsQuery {
            role_code: Some("clerk".into()),
        }))
        .await
        .0;
        let items = clerk
            .get("items")
            .and_then(|v| v.as_array())
            .unwrap()
            .clone();
        assert_eq!(items.len(), 7);
        let academic_mandatory = items
            .iter()
            .find(|r| r["doc_type"] == "academic_cert")
            .map(|r| r["mandatory"].as_bool().unwrap())
            .unwrap();
        assert!(!academic_mandatory);

        // Stable ids across calls.
        let again = handlers::list_document_requirements(Query(ListRequirementsQuery {
            role_code: Some("clerk".into()),
        }))
        .await
        .0;
        assert_eq!(clerk, again);
    }

    #[tokio::test]
    async fn submit_verify_complete_flow() {
        let state = AppState::default();
        let app = Uuid::now_v7();
        let verifier = Uuid::now_v7();

        for doc in [
            "national_id",
            "kra_pin",
            "police_clearance",
            "passport_photo",
        ] {
            let (status, body) =
                handlers::submit_document(State(state.clone()), Json(submit_req(app, doc, "v1")))
                    .await
                    .unwrap();
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(body.0["duplicate"], false);
            let id = body.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

            let res = handlers::record_verdict(
                State(state.clone()),
                Path(id),
                HeaderMap::new(),
                Json(VerdictRequest {
                    verdict: "verified".into(),
                    verifier_id: verifier,
                    reason: None,
                }),
            )
            .await
            .unwrap();
            assert_eq!(res.0["verdict"]["status"], "verified");
        }

        let completeness = handlers::get_application_completeness(
            State(state.clone()),
            Path(app),
            Query(CompletenessQuery {
                role_code: Some("clerk".into()),
            }),
        )
        .await
        .unwrap()
        .0;
        let completeness: DocumentCompleteness = serde_json::from_value(completeness).unwrap();
        assert!(completeness.complete, "{completeness:?}");
        assert!(completeness.missing.is_empty() && completeness.expired.is_empty());
    }

    #[tokio::test]
    async fn duplicate_submission_is_idempotent() {
        let state = AppState::default();
        let app = Uuid::now_v7();

        let first = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "national_id", "same")),
        )
        .await
        .unwrap();
        assert_eq!(first.0, StatusCode::CREATED);
        let original_id = first.1 .0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

        let replay = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "national_id", "same")),
        )
        .await
        .unwrap();
        assert_eq!(replay.0, StatusCode::OK);
        assert_eq!(replay.1 .0["duplicate"], true);
        assert_eq!(
            replay.1 .0["id"].as_str().unwrap().parse::<Uuid>().unwrap(),
            original_id
        );

        // Different hash is a new submission.
        let second = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "national_id", "other")),
        )
        .await
        .unwrap();
        assert_eq!(second.0, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn invalid_hash_rejected() {
        let state = AppState::default();
        let mut req = submit_req(Uuid::now_v7(), "national_id", "x");
        req.content_hash_hex = "not-a-hash".into();
        let err = handlers::submit_document(State(state), Json(req))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            payroll::error::ApiError::Domain(DomainError::InvalidContentHash)
        ));
    }

    #[tokio::test]
    async fn supersede_chain() {
        let state = AppState::default();
        let app = Uuid::now_v7();
        let verifier = Uuid::now_v7();

        let (_, first) = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "passport_photo", "v1")),
        )
        .await
        .unwrap();
        let old_id = first.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

        let _ = handlers::record_verdict(
            State(state.clone()),
            Path(old_id),
            HeaderMap::new(),
            Json(VerdictRequest {
                verdict: "verified".into(),
                verifier_id: verifier,
                reason: None,
            }),
        )
        .await
        .unwrap();

        let (status, second) = handlers::supersede_document(
            State(state.clone()),
            Path(old_id),
            Json(SupersedeDocumentRequest {
                content_hash_hex: sha256_hex("v2"),
                object_key: "tenants/x/passport_photo/v2.pdf".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let new_id = second.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();
        assert_ne!(old_id, new_id);

        // History retained with the chain link set.
        let old = state.documents.get(old_id).unwrap();
        assert_eq!(old.superseded_by, Some(new_id));
        let new = state.documents.get(new_id).unwrap();
        assert_eq!(new.verdict, Verdict::Pending);
        assert_eq!(new.doc_type, "passport_photo");

        // Superseding twice is a conflict.
        let err = handlers::supersede_document(
            State(state.clone()),
            Path(old_id),
            Json(SupersedeDocumentRequest {
                content_hash_hex: sha256_hex("v3"),
                object_key: "tenants/x/passport_photo/v3.pdf".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, payroll::error::ApiError::Conflict(_)));

        // The superseded copy no longer counts toward completeness; only the
        // pending replacement exists, so the gate re-opens as missing.
        let completeness = handlers::get_application_completeness(
            State(state.clone()),
            Path(app),
            Query(CompletenessQuery { role_code: None }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(completeness["complete"], false);
        let missing = completeness["missing"].as_array().unwrap();
        assert!(missing.contains(&Value::String("passport_photo".into())));
    }

    #[tokio::test]
    async fn dual_control_on_national_id_rejection() {
        let state = AppState::default();
        let app = Uuid::now_v7();
        let verifier = Uuid::now_v7();

        let (_, body) = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "national_id", "id")),
        )
        .await
        .unwrap();
        let id = body.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

        // Empty reason => 422.
        let err = handlers::record_verdict(
            State(state.clone()),
            Path(id),
            HeaderMap::new(),
            Json(VerdictRequest {
                verdict: "rejected".into(),
                verifier_id: verifier,
                reason: Some("  ".into()),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            payroll::error::ApiError::Domain(DomainError::RejectionReasonRequired)
        ));

        // Missing second reviewer header => 422.
        let err = handlers::record_verdict(
            State(state.clone()),
            Path(id),
            HeaderMap::new(),
            Json(VerdictRequest {
                verdict: "rejected".into(),
                verifier_id: verifier,
                reason: Some("blurry scan".into()),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            payroll::error::ApiError::Domain(DomainError::DualControlViolation)
        ));

        // Second reviewer equal to the verifier => 422.
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-second-reviewer",
            HeaderValue::from_str(&verifier.to_string()).unwrap(),
        );
        let err = handlers::record_verdict(
            State(state.clone()),
            Path(id),
            headers,
            Json(VerdictRequest {
                verdict: "rejected".into(),
                verifier_id: verifier,
                reason: Some("blurry scan".into()),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            payroll::error::ApiError::Domain(DomainError::DualControlViolation)
        ));

        // Distinct second reviewer passes.
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-second-reviewer",
            HeaderValue::from_str(&Uuid::now_v7().to_string()).unwrap(),
        );
        let ok = handlers::record_verdict(
            State(state.clone()),
            Path(id),
            headers,
            Json(VerdictRequest {
                verdict: "rejected".into(),
                verifier_id: verifier,
                reason: Some("blurry scan".into()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(ok.0["verdict"]["status"], "rejected");
        assert_eq!(ok.0["verdict"]["reason"], "blurry scan");

        // Non-national_id rejection needs no dual control.
        let (_, body) =
            handlers::submit_document(State(state.clone()), Json(submit_req(app, "oath", "oath")))
                .await
                .unwrap();
        let oath_id = body.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();
        let ok = handlers::record_verdict(
            State(state.clone()),
            Path(oath_id),
            HeaderMap::new(),
            Json(VerdictRequest {
                verdict: "rejected".into(),
                verifier_id: verifier,
                reason: Some("unsigned".into()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(ok.0["verdict"]["status"], "rejected");
    }

    #[tokio::test]
    async fn expiry_sweep_detects_regression() {
        let state = AppState::default();
        let regressed_app = Uuid::now_v7();
        let gappy_app = Uuid::now_v7();
        let verifier = Uuid::now_v7();

        // The regressed app verifies every mandatory doc type in the default
        // matrix union; police clearance carries a 12-month validity.
        for doc in [
            "national_id",
            "kra_pin",
            "police_clearance",
            "academic_cert",
            "passport_photo",
        ] {
            let (_, body) = handlers::submit_document(
                State(state.clone()),
                Json(submit_req(regressed_app, doc, doc)),
            )
            .await
            .unwrap();
            let id = body.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();
            let _ = handlers::record_verdict(
                State(state.clone()),
                Path(id),
                HeaderMap::new(),
                Json(VerdictRequest {
                    verdict: "verified".into(),
                    verifier_id: verifier,
                    reason: None,
                }),
            )
            .await
            .unwrap();
        }

        // The gappy app only verifies police clearance: it is already
        // incomplete today, so any future breakage is *missing*, not an
        // expiry regression — it must not be flagged.
        let (_, body) = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(
                gappy_app,
                "police_clearance",
                &gappy_app.to_string(),
            )),
        )
        .await
        .unwrap();
        let id = body.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();
        let _ = handlers::record_verdict(
            State(state.clone()),
            Path(id),
            HeaderMap::new(),
            Json(VerdictRequest {
                verdict: "verified".into(),
                verifier_id: verifier,
                reason: None,
            }),
        )
        .await
        .unwrap();

        // Sweep at +13 months: clearance expired => completeness regressed.
        let as_of = (Utc::now() + Duration::days(396)).to_rfc3339();
        let sweep = handlers::expiry_sweep(
            State(state.clone()),
            Query(ExpirySweepQuery { as_of: Some(as_of) }),
        )
        .await
        .unwrap()
        .0;
        let flagged: Vec<Uuid> = sweep["regressed_application_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        assert_eq!(flagged, vec![regressed_app]);

        // Sweep today finds no regressions.
        let sweep = handlers::expiry_sweep(State(state), Query(ExpirySweepQuery { as_of: None }))
            .await
            .unwrap()
            .0;
        assert!(sweep["regressed_application_ids"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn verify_expiry_and_completeness_are_pure() {
        use payroll::domain::{
            completeness, expiry_regressions, is_valid_content_hash, verify_expiry,
        };

        assert!(is_valid_content_hash(&sha256_hex("abc")));
        assert!(!is_valid_content_hash("deadbeef"));

        let requirement = DocumentRequirement {
            id: Uuid::now_v7(),
            role_code: "po".into(),
            doc_type: "police_clearance".into(),
            mandatory: true,
            validity_months: Some(6),
        };
        let now = Utc::now();
        let submission = DocumentSubmission {
            id: Uuid::now_v7(),
            application_id: Uuid::now_v7(),
            doc_type: "police_clearance".into(),
            content_hash_hex: sha256_hex("pure"),
            object_key: "k".into(),
            channel: "mobile".into(),
            submitted_at: now,
            superseded_by: None,
            verdict: Verdict::Verified {
                verifier_id: Uuid::now_v7(),
                verified_at: now,
            },
        };

        assert!(verify_expiry(&submission, 6, now));
        assert!(!verify_expiry(&submission, 6, now + Duration::days(190)));

        let complete = completeness(
            std::slice::from_ref(&requirement),
            std::slice::from_ref(&submission),
            now,
        );
        assert!(complete.complete);

        let expired = completeness(
            std::slice::from_ref(&requirement),
            std::slice::from_ref(&submission),
            now + Duration::days(200),
        );
        assert!(!expired.complete);
        assert_eq!(expired.expired, vec!["police_clearance".to_string()]);
        assert!(expired.missing.is_empty());

        // Regression requires expiry-only breakage: an unsubmitted mandatory
        // doc must not be reported.
        let never_submitted = DocumentRequirement {
            id: Uuid::now_v7(),
            role_code: "po".into(),
            doc_type: "kra_pin".into(),
            mandatory: true,
            validity_months: None,
        };
        let requirements = vec![never_submitted];
        assert!(expiry_regressions(&requirements, &[submission], Utc::now(),).is_empty());
    }

    #[tokio::test]
    async fn bulk_verdict_applies_and_collects_per_item_errors() {
        let state = AppState::default();
        let app = Uuid::now_v7();
        let verifier = Uuid::now_v7();

        let (_, kra) =
            handlers::submit_document(State(state.clone()), Json(submit_req(app, "kra_pin", "v1")))
                .await
                .unwrap();
        let kra_id = kra.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

        let (_, clearance) = handlers::submit_document(
            State(state.clone()),
            Json(submit_req(app, "police_clearance", "v1")),
        )
        .await
        .unwrap();
        let clearance_id = clearance.0["id"].as_str().unwrap().parse::<Uuid>().unwrap();

        let missing = Uuid::now_v7();

        let res = handlers::bulk_record_verdict(
            State(state.clone()),
            HeaderMap::new(),
            Json(BulkVerdictRequest {
                verdicts: vec![
                    BulkVerdictItem {
                        id: kra_id,
                        verdict: "verified".into(),
                        verifier_id: verifier,
                        reason: None,
                    },
                    BulkVerdictItem {
                        id: clearance_id,
                        verdict: "rejected".into(),
                        verifier_id: verifier,
                        reason: Some("expired".into()),
                    },
                    BulkVerdictItem {
                        id: missing,
                        verdict: "verified".into(),
                        verifier_id: verifier,
                        reason: None,
                    },
                ],
            }),
        )
        .await
        .unwrap()
        .0;

        let results = res["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0]["verdict"]["status"], "verified");
        assert_eq!(results[1]["verdict"]["status"], "rejected");
        assert!(
            results[2].get("error").is_some(),
            "missing submission collects an error"
        );
    }
}

// ---------------------------------------------------------------------------
// Applications: portal intake (design §1)
// ---------------------------------------------------------------------------

mod applications {
    use super::*;
    use payroll::domain::hash_sha256_hex;
    use payroll::handlers::{CompletenessQuery, CreateApplicationRequest, NewDocument};

    fn req(name: &str, national_id: &str) -> CreateApplicationRequest {
        CreateApplicationRequest {
            applicant_name: name.into(),
            national_id: national_id.into(),
            role_code: "clerk".into(),
            county: "KE-029".into(),
            phone: Some("0712345678".into()),
            email: None,
            documents: vec![NewDocument {
                doc_type: "national_id".into(),
                content_hash_hex: hash_sha256_hex("id-scan"),
                object_key: "tenants/app/id.pdf".into(),
                channel: "web".into(),
            }],
        }
    }

    #[tokio::test]
    async fn intake_hashes_pii_and_persists_documents() {
        let state = AppState::default();
        let (status, body) = handlers::create_application(
            State(state.clone()),
            Json(req("Wanjiku Njeri", "12345678")),
        )
        .await
        .unwrap();

        assert_eq!(status, StatusCode::CREATED);
        let app = &body.0["application"];
        assert_eq!(app["applicant_name"], "Wanjiku Njeri");
        assert_eq!(app["national_id_hash"], hash_sha256_hex("12345678"));
        assert_eq!(app["contact_phone_hash"], hash_sha256_hex("0712345678"));
        assert!(app["application_ref"].as_str().unwrap().starts_with("PWP-"));
        // The raw national ID must never appear in the stored application.
        assert!(app.get("national_id").is_none());
        assert_eq!(app["status"], "applied");

        // Document metadata is persisted against the application id.
        let app_id = app["id"].as_str().unwrap().parse::<Uuid>().unwrap();
        let completeness = handlers::get_application_completeness(
            State(state.clone()),
            Path(app_id),
            Query(CompletenessQuery {
                role_code: Some("clerk".into()),
            }),
        )
        .await
        .unwrap()
        .0;
        let completeness: DocumentCompleteness = serde_json::from_value(completeness).unwrap();
        // national_id submitted but still pending verification -> incomplete.
        assert!(!completeness.complete);

        // Lookup by reference returns the same application.
        let reference = app["application_ref"].as_str().unwrap().to_string();
        let found = handlers::get_application_by_reference(State(state), Path(reference))
            .await
            .unwrap()
            .0;
        assert_eq!(found["applicant_name"], "Wanjiku Njeri");
    }

    #[tokio::test]
    async fn intake_is_idempotent_on_national_id() {
        let state = AppState::default();
        let first =
            handlers::create_application(State(state.clone()), Json(req("Ada Oburu", "87654321")))
                .await
                .unwrap();
        assert_eq!(first.0, StatusCode::CREATED);
        let original_ref = first.1 .0["application"]["application_ref"]
            .as_str()
            .unwrap()
            .to_string();

        let replay = handlers::create_application(State(state), Json(req("Ada Oburu", "87654321")))
            .await
            .unwrap();
        assert_eq!(replay.0, StatusCode::OK);
        assert_eq!(replay.1 .0["duplicate"], true);
        assert_eq!(
            replay.1 .0["application"]["application_ref"]
                .as_str()
                .unwrap(),
            original_ref
        );
    }

    #[tokio::test]
    async fn intake_rejects_missing_required_fields() {
        let state = AppState::default();
        let mut bad = req("", "12345678");
        bad.national_id = String::new();
        let err = handlers::create_application(State(state), Json(bad))
            .await
            .unwrap_err();
        assert!(matches!(err, payroll::error::ApiError::BadRequest(_)));
    }

    #[tokio::test]
    async fn lookup_by_unknown_reference_is_not_found() {
        let state = AppState::default();
        let err =
            handlers::get_application_by_reference(State(state), Path("PWP-NOPE-0000".into()))
                .await
                .unwrap_err();
        assert!(matches!(err, payroll::error::ApiError::NotFound));
    }
}
