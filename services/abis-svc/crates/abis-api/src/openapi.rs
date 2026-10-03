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

//! OpenAPI specification exporter.
//!
//! Serves:
//! - GET /openapi.json  -> raw OpenAPI 3.0 JSON
//! - GET /docs          -> Swagger UI

use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::{
    models::{
        AbisReport, AdjudicationCase, AdjudicationCasePage, AdjudicationResolutionRequest,
        AdjudicationStatus, BatchDedupRequest, BatchJobAccepted, HealthResponse,
        ListAdjudicationQuery, Match11Request, Match11Result, Match1NRequest, Match1NResult,
        MatchAccepted, MatchCandidate, MatchOutcome, Modality,
    },
    routes::{
        adjudication::{AssignRequest, DecideRequest, EscalateRequest, EvidenceItem},
        admin::{AttestationPolicyUpdate, PolicyUpdateResult, VendorHealthCheckResult, VendorInfo},
        enrolment::{EnrolmentJobAccepted, EnrolmentJobRequest, EnrolmentJobStatus, QualityReport},
        health::{HealthResponse as HealthR, ReadinessResponse},
        kit::{GalleryConfig, KitEnrolRequest, KitTemplate, KitVerifyRequest, KitVerifyResponse},
        match_routes::{BatchMatchAccepted, BatchMatchRequest},
        reports::{DedupSummary, FairnessGroup, FairnessReport, FmrFnmrReport, ThroughputReport},
    },
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "EEL ABIS API",
        version = "1.0.0",
        description = "Automated Biometric Identification System REST API"
    ),
    paths(
        crate::routes::enrolment::create_enrolment_job,
        crate::routes::enrolment::get_enrolment_job,
        crate::routes::enrolment::withdraw_enrolment_job,
        crate::routes::enrolment::get_quality_report,
        crate::routes::match_routes::match_1n,
        crate::routes::match_routes::match_11,
        crate::routes::match_routes::match_batch,
        crate::routes::match_routes::get_match_result,
        crate::routes::adjudication::list_adjudication,
        crate::routes::adjudication::assign_adjudication,
        crate::routes::adjudication::decide_adjudication,
        crate::routes::adjudication::escalate_adjudication,
        crate::routes::adjudication::list_evidence,
        crate::routes::adjudication::upload_evidence,
        crate::routes::kit::kit_enrol,
        crate::routes::kit::kit_verify,
        crate::routes::kit::kit_gallery_config,
        crate::routes::reports::get_fmr_fnmr,
        crate::routes::reports::get_dedup_summary,
        crate::routes::reports::get_fairness,
        crate::routes::reports::get_enrolment_throughput,
        crate::routes::admin::update_policy,
        crate::routes::admin::list_vendors,
        crate::routes::admin::vendor_health_check,
        crate::routes::health::healthz,
        crate::routes::health::readyz,
    ),
    components(
        schemas(
            EnrolmentJobRequest, EnrolmentJobAccepted, EnrolmentJobStatus, QualityReport,
            Match1NRequest, Match11Request, MatchAccepted, Match11Result, Match1NResult,
            MatchCandidate, Modality, MatchOutcome, BatchMatchRequest, BatchMatchAccepted,
            AdjudicationCase, AdjudicationCasePage, AdjudicationResolutionRequest,
            AdjudicationStatus, ListAdjudicationQuery, AssignRequest, DecideRequest,
            EscalateRequest, EvidenceItem,
            KitEnrolRequest, KitTemplate, KitVerifyRequest, KitVerifyResponse, GalleryConfig,
            FmrFnmrReport, DedupSummary, FairnessReport, FairnessGroup, ThroughputReport,
            AbisReport, HealthResponse,
            AttestationPolicyUpdate, PolicyUpdateResult, VendorInfo, VendorHealthCheckResult,
            HealthR, ReadinessResponse,
            BatchDedupRequest, BatchJobAccepted,
        )
    ),
    tags(
        (name = "enrolment", description = "Enrolment job management"),
        (name = "match", description = "Biometric matching"),
        (name = "adjudication", description = "Adjudication case management"),
        (name = "kit", description = "BVR/BVI kit endpoints"),
        (name = "reports", description = "Analytics and fairness reports"),
        (name = "admin", description = "Administrative operations"),
        (name = "health", description = "Health probes"),
    )
)]
pub struct ApiDoc;

/// GET /openapi.json
pub async fn serve_openapi(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<utoipa::openapi::OpenApi>) {
    (StatusCode::OK, Json(ApiDoc::openapi()))
}
