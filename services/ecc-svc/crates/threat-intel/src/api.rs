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

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    correlator::{CorrelationMatch, Correlator, EccFlag},
    feed::{FeedManager, TaxiiEndpoint},
    stix::StixBundle,
};

#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<FeedManager>,
    pub correlator: Arc<Correlator>,
    pub endpoints: Arc<RwLock<Vec<TaxiiEndpoint>>>,
    pub matches: Arc<RwLock<Vec<CorrelationMatch>>>,
}

impl AppState {
    pub fn new(endpoints: Vec<TaxiiEndpoint>) -> Self {
        let bundles: Arc<RwLock<Vec<StixBundle>>> = Arc::new(RwLock::new(Vec::new()));
        let manager = Arc::new(FeedManager {
            client: reqwest::Client::new(),
            endpoints: endpoints.clone(),
            bundles: bundles.clone(),
        });
        let correlator = Arc::new(Correlator::new(bundles));
        Self {
            manager,
            correlator,
            endpoints: Arc::new(RwLock::new(endpoints)),
            matches: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
struct FeedInfo {
    label: String,
    url: String,
}

async fn list_feeds(State(state): State<AppState>) -> impl IntoResponse {
    let eps = state.endpoints.read().await;
    let infos: Vec<FeedInfo> = eps
        .iter()
        .map(|e| FeedInfo {
            label: e.label.clone(),
            url: e.url.clone(),
        })
        .collect();
    Json(infos)
}

async fn add_feed(State(state): State<AppState>, Json(ep): Json<AddFeedBody>) -> impl IntoResponse {
    let mut eps = state.endpoints.write().await;
    eps.push(TaxiiEndpoint {
        url: ep.url,
        collection_id: ep.collection_id,
        api_root: ep.api_root,
        label: ep.label,
    });
    StatusCode::CREATED
}

#[derive(Debug, Deserialize)]
struct AddFeedBody {
    url: String,
    collection_id: String,
    api_root: String,
    label: String,
}

async fn remove_feed(
    State(state): State<AppState>,
    Path(label): Path<String>,
) -> impl IntoResponse {
    let mut eps = state.endpoints.write().await;
    let before = eps.len();
    eps.retain(|e| e.label != label);
    if eps.len() < before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Debug, Deserialize)]
struct Pagination {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_per_page")]
    per_page: usize,
}

fn default_page() -> usize {
    1
}
fn default_per_page() -> usize {
    20
}

async fn list_matches(
    State(state): State<AppState>,
    Query(paging): Query<Pagination>,
) -> impl IntoResponse {
    let all = state.matches.read().await;
    let start = (paging.page.saturating_sub(1)) * paging.per_page;
    let page: Vec<CorrelationMatch> = all
        .iter()
        .skip(start)
        .take(paging.per_page)
        .cloned()
        .collect();
    Json(page)
}

async fn correlate_flag(
    State(state): State<AppState>,
    Json(flag): Json<EccFlag>,
) -> impl IntoResponse {
    let results = state.correlator.correlate(&flag).await;
    Json(results)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/threat-intel/feeds", get(list_feeds).post(add_feed))
        .route("/v1/threat-intel/feeds/:label", delete(remove_feed))
        .route("/v1/threat-intel/matches", get(list_matches))
        .route("/v1/threat-intel/correlate", post(correlate_flag))
        .with_state(state)
}
