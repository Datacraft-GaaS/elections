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

use axum::{
	body::Body,
	extract::Request,
	middleware::Next,
	response::Response,
};
use eel_auth_rs::JwksVerifier;
use crate::problem::Problem;

/// Axum middleware that validates the Bearer JWT via JWKS.
///
/// Injects `Subject` into request extensions on success.
/// Returns RFC 9457 401 without a valid token.
pub async fn require_jwt(
	jwks: std::sync::Arc<JwksVerifier>,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	let token = req
		.headers()
		.get(http::header::AUTHORIZATION)
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.strip_prefix("Bearer "));

	match token {
		None => Problem::unauthorized("Missing Bearer token").to_axum_response(),
		Some(tok) => match jwks.verify(tok).await {
			Ok(subject) => {
				req.extensions_mut().insert(subject);
				next.run(req).await
			}
			Err(_) => Problem::unauthorized("Invalid or expired token").to_axum_response(),
		},
	}
}
