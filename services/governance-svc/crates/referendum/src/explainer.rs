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

//! Referendum explainer public contract (explainer/referendum.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::SharedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumExplainerCreate {
    pub question_id: String,
    pub question_text_en: String,
    pub question_text_sw: String,
    pub explanation_en: String,
    pub explanation_sw: String,
    pub gazette_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumExplainer {
    pub question_id: String,
    pub question_text_en: String,
    pub question_text_sw: String,
    pub explanation_en: String,
    pub explanation_sw: String,
    pub gazette_ref: String,
    pub implication_en: Option<String>,
    pub implication_sw: Option<String>,
    pub faq: Option<Vec<FaqItem>>,
    pub legal_reference: Option<String>,
    pub media_refs: Option<Vec<MediaRef>>,
    pub published_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqItem {
    pub q: String,
    pub a: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRef {
    pub r#type: String,
    pub url: String,
    pub caption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumExplainerSummary {
    pub question_id: String,
    pub question_text_en: String,
    pub question_text_sw: String,
    pub published_at: String,
}

/// GET /api/v1/explainer/referendum/questions — list explainer summaries.
pub async fn list_referendum_explainers(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.explainers.lock().expect("lock poisoned");
    let mut items: Vec<ReferendumExplainerSummary> = guard
        .values()
        .map(|e| ReferendumExplainerSummary {
            question_id: e.question_id.clone(),
            question_text_en: e.question_text_en.clone(),
            question_text_sw: e.question_text_sw.clone(),
            published_at: e.published_at.clone(),
        })
        .collect();
    drop(guard);
    items.sort_by(|a, b| a.question_id.cmp(&b.question_id));
    Json(serde_json::json!({ "questions": items, "total": items.len() }))
}

/// POST /api/v1/explainer/referendum/questions — publish an explainer.
pub async fn publish_referendum_explainer(
    State(state): State<SharedState>,
    Json(req): Json<ReferendumExplainerCreate>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let explainer = ReferendumExplainer {
        question_id: req.question_id,
        question_text_en: req.question_text_en,
        question_text_sw: req.question_text_sw,
        explanation_en: req.explanation_en,
        explanation_sw: req.explanation_sw,
        gazette_ref: req.gazette_ref,
        implication_en: None,
        implication_sw: None,
        faq: None,
        legal_reference: None,
        media_refs: None,
        published_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.explainers.lock().expect("lock poisoned");
    guard.insert(explainer.question_id.clone(), explainer.clone());
    drop(guard);
    (StatusCode::CREATED, Json(explainer))
}

/// GET /api/v1/explainer/referendum/questions/{question_id} — fetch an explainer.
pub async fn get_referendum_explainer(
    State(state): State<SharedState>,
    Path(question_id): Path<String>,
) -> impl IntoResponse {
    let guard = state.explainers.lock().expect("lock poisoned");
    match guard.get(&question_id) {
        Some(e) => Json(e.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "explainer not found" })),
        )
            .into_response(),
    }
}

/// GET /api/v1/explainer/referendum/questions/{question_id}/translations —
/// list translations for a question.
pub async fn get_referendum_explainer_translations(
    State(state): State<SharedState>,
    Path(question_id): Path<String>,
) -> impl IntoResponse {
    let guard = state.explainers.lock().expect("lock poisoned");
    match guard.get(&question_id) {
        Some(e) => {
            let translations = serde_json::json!([
                { "language_code": "en", "language_name": "English", "question_text": e.question_text_en, "explanation": e.explanation_en },
                { "language_code": "sw", "language_name": "Kiswahili", "question_text": e.question_text_sw, "explanation": e.explanation_sw },
            ]);
            Json(serde_json::json!({ "question_id": question_id, "translations": translations, "total_languages": 2 }))
				.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "explainer not found" })),
        )
            .into_response(),
    }
}
