//! Public contract handlers (field-mobile-support/support.yaml).

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};

use crate::spec;
use crate::state::AppState;

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_cursor_value(items)
}

/// POST /api/v1/mobile/support/tickets — create a support ticket.
pub async fn create_support_ticket(
	State(state): State<AppState>,
	Json(req): Json<spec::SupportTicketRequest>,
) -> impl IntoResponse {
	let now = chrono::Utc::now().to_rfc3339();
	let ticket = spec::SupportTicket {
		ticket_id: uuid::Uuid::now_v7().to_string(),
		station_code: req.station_code,
		kit_id_hash: req.kit_id_hash,
		issue_type: req.issue_type,
		description: req.description,
		severity: req.severity,
		status: "open".to_string(),
		resolution_notes: None,
		resolved_by_hash: None,
		device_info: req.device_info.unwrap_or_default(),
		created_at: now.clone(),
		updated_at: now,
		resolved_at: None,
	};
	state
		.spec_tickets
		.lock()
		.expect("lock poisoned")
		.insert(ticket.ticket_id.clone(), ticket.clone());
	(StatusCode::CREATED, Json(ticket))
}

/// GET /api/v1/mobile/support/tickets — list support tickets.
pub async fn list_support_tickets(State(state): State<AppState>) -> impl IntoResponse {
	let guard = state.spec_tickets.lock().expect("lock poisoned");
	let mut items: Vec<spec::SupportTicket> = guard.values().cloned().collect();
	drop(guard);
	items.sort_by(|a, b| a.ticket_id.cmp(&b.ticket_id));
	Json(page(&serde_json::to_value(items).unwrap()))
}

/// GET /api/v1/mobile/support/tickets/{ticket_id} — fetch one ticket.
pub async fn get_support_ticket(
	State(state): State<AppState>,
	Path(ticket_id): Path<String>,
) -> impl IntoResponse {
	let guard = state.spec_tickets.lock().expect("lock poisoned");
	match guard.get(&ticket_id) {
		Some(t) => Json(t.clone()).into_response(),
		None => (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({ "detail": "ticket not found" })),
		)
			.into_response(),
	}
}

/// PATCH /api/v1/mobile/support/tickets/{ticket_id} — update a ticket.
pub async fn update_support_ticket(
	State(state): State<AppState>,
	Path(ticket_id): Path<String>,
	Json(req): Json<spec::TicketUpdate>,
) -> impl IntoResponse {
	let mut guard = state.spec_tickets.lock().expect("lock poisoned");
	match guard.get_mut(&ticket_id) {
		Some(t) => {
			if let Some(v) = req.status { t.status = v; }
			if let Some(v) = req.resolution_notes { t.resolution_notes = Some(v); }
			if let Some(v) = req.resolved_by_hash { t.resolved_by_hash = Some(v); }
			t.updated_at = chrono::Utc::now().to_rfc3339();
			Json(t.clone()).into_response()
		}
		None => (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({ "detail": "ticket not found" })),
		)
			.into_response(),
	}
}

/// GET /api/v1/mobile/support/knowledge-base — list article summaries.
pub async fn list_knowledge_base_articles(State(state): State<AppState>) -> impl IntoResponse {
	let guard = state.spec_articles.lock().expect("lock poisoned");
	let mut items: Vec<spec::ArticleSummary> = guard
		.values()
		.map(|a| spec::ArticleSummary {
			article_id: a.article_id.clone(),
			title: a.title.clone(),
			category: a.category.clone(),
			summary: a.body.chars().take(120).collect(),
			version: a.version,
			updated_at: a.updated_at.clone(),
		})
		.collect();
	drop(guard);
	items.sort_by(|a, b| a.article_id.cmp(&b.article_id));
	Json(page(&serde_json::to_value(items).unwrap()))
}

/// GET /api/v1/mobile/support/knowledge-base/{article_id} — fetch one article.
pub async fn get_knowledge_base_article(
	State(state): State<AppState>,
	Path(article_id): Path<String>,
) -> impl IntoResponse {
	let guard = state.spec_articles.lock().expect("lock poisoned");
	match guard.get(&article_id) {
		Some(a) => Json(a.clone()).into_response(),
		None => (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({ "detail": "article not found" })),
		)
			.into_response(),
	}
}
