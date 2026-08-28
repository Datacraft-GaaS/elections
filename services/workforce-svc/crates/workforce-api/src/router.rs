use crate::handlers::*;
use crate::store::Store;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router(store: Store) -> Router {
    Router::new()
        .route("/health", get(health))
        // Registry & contacts
        .route(
            "/v1/workforce/workers",
            get(list_workers).post(create_worker),
        )
        .route(
            "/v1/workforce/workers/{id}",
            get(get_worker).put(update_worker),
        )
        .route("/v1/workforce/workers/{id}/badge", get(get_badge))
        .route(
            "/v1/workforce/workers/{id}/contacts/{channel}",
            put(upsert_contact),
        )
        .route(
            "/v1/workforce/workers/{id}/contacts/{channel}/verify",
            post(verify_contact),
        )
        .route("/v1/workforce/workers/{id}/contacts", get(list_contacts))
        .route(
            "/v1/workforce/workers/{id}/contact-history",
            get(contact_history),
        )
        // Recruitment state machine
        .route(
            "/v1/workforce/recruitment",
            get(list_cases).post(create_case),
        )
        .route("/v1/workforce/recruitment/{id}", get(get_case))
        .route(
            "/v1/workforce/recruitment/{id}/transition",
            post(transition_case),
        )
        .route(
            "/v1/workforce/recruitment/{id}/interviews",
            get(list_interviews).post(create_interview),
        )
        // Postings & equipment claims
        .route(
            "/v1/workforce/postings",
            get(list_postings).post(create_posting),
        )
        .route("/v1/workforce/postings/bulk", post(bulk_create_postings))
        .route(
            "/v1/workforce/postings/{id}/supersede",
            post(supersede_posting),
        )
        .route(
            "/v1/workforce/postings/{id}/equipment",
            post(create_equipment_claim),
        )
        .route(
            "/v1/workforce/equipment/{claim_id}/acknowledge",
            post(acknowledge_claim),
        )
        .route(
            "/v1/workforce/equipment/{claim_id}/return",
            post(return_claim),
        )
        .route(
            "/v1/workforce/equipment/{claim_id}/flag-loss",
            post(flag_loss),
        )
        // Tasking
        .route(
            "/v1/workforce/task-templates",
            get(list_templates).post(create_template),
        )
        .route("/v1/workforce/tasks", get(list_tasks).post(assign_task))
        .route("/v1/workforce/tasks/{id}/ack", post(ack_task))
        .route("/v1/workforce/tasks/{id}/done", post(done_task))
        .route("/v1/workforce/tasks/{id}/blocked", post(block_task))
        // Performance
        .route(
            "/v1/workforce/performance",
            get(get_performance).post(add_record),
        )
        // Notifications
        .route(
            "/v1/workforce/notifications",
            get(list_notifications).post(send_notification),
        )
        // Operations dashboard rollups
        .route("/v1/workforce/readiness", get(readiness))
        .route("/v1/workforce/audit", get(audit_trail))
        .with_state(store)
}
