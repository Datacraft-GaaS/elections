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

#[tokio::test]
async fn health_returns_ok() {
    let app = rts_api::router::build_router(Arc::new(rts_api::store::AppState::default()));
    let resp = app.oneshot(http_request()).await.unwrap();
    assert_eq!(resp.status(), 200);
}

fn http_request() -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap()
}

use tower::ServiceExt;
