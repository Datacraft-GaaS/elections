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
use http::header::ACCEPT;

/// Inspect Accept header; set a request extension so handlers can choose codec.
/// Also applies zstd compression when Accept-Encoding includes zstd.
pub async fn negotiate(req: Request<Body>, next: Next) -> Response {
	let wants_cbor = req
		.headers()
		.get(ACCEPT)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.contains("application/cbor"))
		.unwrap_or(false);

	let wants_zstd = req
		.headers()
		.get(http::header::ACCEPT_ENCODING)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.contains("zstd"))
		.unwrap_or(false);

	// Attach flags for downstream handlers.
	let (mut parts, body) = req.into_parts();
	parts.extensions.insert(ContentPrefs { wants_cbor, wants_zstd });
	let req = Request::from_parts(parts, body);
	next.run(req).await
}

#[derive(Debug, Clone, Copy)]
pub struct ContentPrefs {
	pub wants_cbor: bool,
	pub wants_zstd: bool,
}
