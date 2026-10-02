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

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use eel_resilience_rs::DegradationFlag;
use crate::problem::Problem;

/// Reject mutating requests (POST/PUT/PATCH/DELETE) when in ReadOnlyLockdown.
pub async fn guard_writes(
	flag: std::sync::Arc<DegradationFlag>,
	req: Request<Body>,
	next: Next,
) -> Response {
	use http::Method;
	let method = req.method().clone();
	let is_write = matches!(method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE);
	if is_write && !flag.is_write_allowed().await {
		return Problem::write_locked().to_axum_response();
	}
	next.run(req).await
}
