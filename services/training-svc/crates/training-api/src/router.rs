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

use crate::handlers::{accreditation, cms, exam, health, lms, medical, spec};
use axum::{
    routing::{get, patch, post, put},
    Router,
};

pub fn build_router() -> Router {
    // Exam engine (§3.2) runs behind its own state; merged with the rest of
    // the training surface which is currently stateless.
    let app = Router::new()
        .route("/healthz", get(health))
        // CMS (M37)
        .route(
            "/v1/training/cms/content",
            post(cms::post_content).get(cms::get_content_list),
        )
        .route(
            "/v1/training/cms/content/public",
            get(cms::get_content_public),
        )
        .route(
            "/v1/training/cms/content/{id}/fact-check",
            patch(cms::submit_fact_check),
        )
        .route(
            "/v1/training/cms/content/{id}/approve",
            post(cms::approve_content),
        )
        // LMS (M38)
        .route(
            "/v1/training/lms/modules",
            post(lms::post_module).get(lms::get_modules),
        )
        .route(
            "/v1/training/lms/modules/{id}/assessments",
            post(lms::submit_assessment),
        )
        .route(
            "/v1/training/lms/certificates/{worker_id}",
            get(lms::get_certificate),
        )
        .route(
            "/v1/training/lms/certificates/verify/{qr_token}",
            get(lms::verify_certificate_qr),
        )
        .route(
            "/v1/training/lms/certificates/{id}/revoke",
            post(lms::revoke_certificate),
        )
        .route("/v1/training/lms/progress", get(lms::get_progress))
        .route(
            "/v1/training/lms/briefing/{worker_id}/confirm",
            post(lms::confirm_briefing),
        )
        // Accreditation (M42+M43)
        .route(
            "/v1/training/accreditations",
            post(accreditation::submit_media_application),
        )
        .route(
            "/v1/training/badges/{serial}/verify",
            get(accreditation::verify_badge),
        )
        .route("/v1/training/crl", get(accreditation::get_crl))
        .route(
            "/v1/training/accreditations/{id}/revoke",
            post(accreditation::revoke_badge),
        )
        .route(
            "/v1/training/organisations/{org_id}/observers",
            post(accreditation::submit_observer_application),
        )
        .route(
            "/v1/training/public/summary",
            get(accreditation::get_public_summary),
        )
        // Medical (M60)
        .route(
            "/v1/training/medical/incidents",
            post(medical::report_incident).get(medical::list_incidents),
        )
        .route(
            "/v1/training/medical/incidents/{id}/dispatch",
            post(medical::dispatch_incident),
        )
        .route(
            "/v1/training/medical/incidents/{id}/resolve",
            put(medical::resolve_incident),
        )
        .route(
            "/v1/training/medical/cluster-alerts",
            get(medical::list_cluster_alerts),
        )
        .route(
            "/v1/training/medical/cluster-alerts/{id}/resolve",
            post(medical::resolve_cluster_alert),
        );
    // One exam store for both surfaces, so the question bank `submit_assessment`
    // scores against is the bank the exam module serves.
    let exam = exam::ExamApiState::new();
    app.merge(spec::router(crate::spec_state::SpecState::with_exam(
        exam.clone(),
    )))
    .merge(exam::router(exam))
}
