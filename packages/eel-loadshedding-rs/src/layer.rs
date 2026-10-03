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

use crate::{priority::Priority, shedder::CoDelShedder};
use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

/// Tower layer that applies CoDel load shedding.
#[derive(Clone)]
pub struct LoadSheddingLayer {
    shedder: Arc<CoDelShedder>,
}

impl LoadSheddingLayer {
    pub fn new(shedder: CoDelShedder) -> Self {
        Self {
            shedder: Arc::new(shedder),
        }
    }

    pub fn metrics(&self) -> &crate::metrics::ShedMetrics {
        &self.shedder.metrics
    }
}

impl<S> Layer<S> for LoadSheddingLayer {
    type Service = LoadSheddingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoadSheddingService {
            inner,
            shedder: self.shedder.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoadSheddingService<S> {
    inner: S,
    shedder: Arc<CoDelShedder>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoadSheddingService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: Send + 'static,
    ResBody: http_body::Body<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let priority = Priority::from_headers(req.headers());
        let shedder = self.shedder.clone();
        let mut inner = self.inner.clone();
        let now = Instant::now();

        Box::pin(async move {
            match shedder.admit(priority, now).await {
                Ok(_permit) => {
                    // Forward to inner service
                    let resp = inner.call(req).await.map_err(Into::into)?;
                    // Collect response body into Bytes
                    let (parts, body) = resp.into_parts();
                    use http_body_util::BodyExt;
                    let bytes = body
                        .collect()
                        .await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?
                        .to_bytes();
                    Ok(Response::from_parts(parts, Full::new(bytes)))
                }
                Err(retry_after) => {
                    // 503 with Retry-After and problem+json
                    let body = serde_json::json!({
                        "type": "https://eel.internal/problems/overloaded",
                        "title": "Service Overloaded",
                        "status": 503,
                        "detail": format!("Priority class '{}' shed under load", priority.name()),
                        "retry_after": retry_after,
                    });
                    let json = serde_json::to_string(&body).unwrap_or_default();
                    let resp = Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .header("Content-Type", "application/problem+json")
                        .header("Retry-After", retry_after.to_string())
                        .body(Full::new(Bytes::from(json)))
                        .unwrap();
                    Ok(resp)
                }
            }
        })
    }
}
