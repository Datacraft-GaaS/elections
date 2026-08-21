use crate::handlers::{accreditation, cms, health, lms, medical};
use axum::{
    routing::{get, patch, post, put},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
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
        )
}
