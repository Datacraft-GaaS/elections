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
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn inject_request_id(mut req: Request<Body>, next: Next) -> Response {
	let id = req
		.headers()
		.get(REQUEST_ID_HEADER)
		.and_then(|v| v.to_str().ok())
		.map(str::to_owned)
		.unwrap_or_else(|| Uuid::new_v4().to_string());

	req.extensions_mut().insert(RequestId(id.clone()));

	let mut resp = next.run(req).await;
	resp.headers_mut()
		.insert(REQUEST_ID_HEADER, id.parse().unwrap());
	resp
}

#[derive(Clone, Debug)]
pub struct RequestId(pub String);
