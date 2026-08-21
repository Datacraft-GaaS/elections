use axum::{http::StatusCode, routing::post, Json, Router};
use bvr_batch::ingest::{process_batch, IngestRow};
use bvr_domain::voter::BiometricTemplateRef;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct EnrolmentRequest {
    pub idempotency_key: String,
    pub national_id: String,
    pub full_name: String,
    pub dob: String,
    pub county_code: String,
    pub polling_station_id: String,
    pub election_scope: String,
    pub biometric_template_refs: Option<Vec<BiometricTemplateRef>>,
}

#[derive(Debug, Serialize)]
pub struct EnrolmentAccepted {
    pub voter_id: String,
    pub status: String,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/bvr/enrolments", post(create_enrolment))
}

async fn create_enrolment(
    Json(req): Json<EnrolmentRequest>,
) -> Result<(StatusCode, Json<EnrolmentAccepted>), ApiError> {
    let row = IngestRow {
        idempotency_key: req.idempotency_key,
        national_id: crate::nfc::nfc(req.national_id),
        full_name: crate::nfc::nfc(req.full_name),
        dob: req.dob,
        county_code: req.county_code,
        polling_station_id: req.polling_station_id,
        election_scope: req.election_scope,
        biometric_template_refs: req.biometric_template_refs.unwrap_or_default(),
    };
    let result = process_batch(vec![row]).await;
    if let Some(outcome) = result.outcomes.into_iter().next() {
        if outcome.success {
            return Ok((
                StatusCode::ACCEPTED,
                Json(EnrolmentAccepted {
                    voter_id: outcome.voter_id.unwrap_or_default(),
                    status: "accepted".into(),
                }),
            ));
        } else {
            return Err(ApiError::BadRequest(outcome.error.unwrap_or_default()));
        }
    }
    Err(ApiError::Internal("empty batch result".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode as Sc},
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn create_enrolment_returns_202_for_valid_request() {
        let app = router();
        let body = serde_json::json!({
            "idempotency_key": "test-key-1",
            "national_id": "KE12345678",
            "full_name": "Jane Voter",
            "dob": "1990-06-15",
            "county_code": "047",
            "polling_station_id": "047-001-001",
            "election_scope": "2027-general"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bvr/enrolments")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), Sc::ACCEPTED);
    }

    #[tokio::test]
    async fn create_enrolment_returns_422_for_missing_fields() {
        let app = router();
        let body = serde_json::json!({"national_id": "KE12345678"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bvr/enrolments")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // axum returns 422 for deserialization failures on missing required fields
        assert_eq!(resp.status(), Sc::UNPROCESSABLE_ENTITY);
    }
}
