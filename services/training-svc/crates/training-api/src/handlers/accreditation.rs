// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::models::*;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::info;
use uuid::Uuid;

pub async fn submit_media_application(Json(req): Json<MediaAccreditationReq>) -> impl IntoResponse {
    info!(organisation_id=%req.organisation_id, "submit_media_application");
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": Uuid::new_v4(),
            "organisation_id": req.organisation_id,
            "access_zones": req.access_zones,
            "status": "pending"
        })),
    )
}

pub async fn verify_badge(Path(serial): Path<String>) -> impl IntoResponse {
    info!(serial, "verify_badge");
    // No PII: applicant_name and id_number are not included in response
    Json(BadgeVerifyResp {
        serial,
        status: "valid".to_string(),
        access_zones: vec![],
    })
}

pub async fn get_crl() -> impl IntoResponse {
    Json(CrlResp {
        generated_at: "2027-08-09T06:00:00Z".to_string(),
        revoked_serials: vec![],
    })
}

pub async fn revoke_badge(Path(id): Path<Uuid>) -> impl IntoResponse {
    info!(accreditation_id=%id, "revoke_badge");
    Json(serde_json::json!({ "id": id, "revoked": true }))
}

pub async fn submit_observer_application(
    Path(org_id): Path<Uuid>,
    Json(req): Json<ObserverApplicationReq>,
) -> impl IntoResponse {
    info!(org_id=%org_id, full_name=%req.full_name, "submit_observer_application");
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": Uuid::new_v4(),
            "organisation_id": org_id,
            "deployment_county_codes": req.deployment_county_codes,
            "status": "pending"
        })),
    )
}

pub async fn get_public_summary() -> impl IntoResponse {
    Json(serde_json::json!({
        "accredited_media_count": 0,
        "accredited_observer_count": 0,
        "as_of": "2027-08-09T06:00:00Z"
    }))
}
