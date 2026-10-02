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

use crate::error::ApiServerError;
use crate::server::ApiServer;
use eel_auth_rs::JwksVerifier;
use eel_loadshedding_rs::{CoDelConfig, CoDelShedder};
use eel_observability_rs::OtelConfig;
use eel_resilience_rs::{DegradationFlag, IdempotencyStore};
use std::sync::Arc;

/// Fluent builder for `ApiServer`.
///
/// Required: `jwks_url`, `otel_service_name`.
/// Everything else has sensible defaults.
pub struct ApiServerBuilder {
    jwks_url: Option<String>,
    jwks_realm: Option<String>,
    otel_service_name: Option<String>,
    degradation_flag: DegradationFlag,
    idempotency_ttl_secs: i64,
    codel_config: CoDelConfig,
    bind_addr: String,
}

impl Default for ApiServerBuilder {
    fn default() -> Self {
        Self {
            jwks_url: None,
            jwks_realm: None,
            otel_service_name: None,
            degradation_flag: DegradationFlag::new(),
            idempotency_ttl_secs: 86_400,
            codel_config: CoDelConfig::default(),
            bind_addr: "0.0.0.0:8080".to_owned(),
        }
    }
}

impl ApiServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jwks_url(mut self, url: impl Into<String>) -> Self {
        self.jwks_url = Some(url.into());
        self
    }

    pub fn jwks_realm(mut self, realm: impl Into<String>) -> Self {
        self.jwks_realm = Some(realm.into());
        self
    }

    pub fn otel_service_name(mut self, name: impl Into<String>) -> Self {
        self.otel_service_name = Some(name.into());
        self
    }

    pub fn degradation_flag(mut self, flag: DegradationFlag) -> Self {
        self.degradation_flag = flag;
        self
    }

    pub fn idempotency_ttl_secs(mut self, secs: i64) -> Self {
        self.idempotency_ttl_secs = secs;
        self
    }

    pub fn codel_config(mut self, cfg: CoDelConfig) -> Self {
        self.codel_config = cfg;
        self
    }

    pub fn bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    pub fn build(self) -> Result<ApiServer, ApiServerError> {
        let jwks_url = self.jwks_url.unwrap_or_else(|| {
            std::env::var("JWKS_URL").unwrap_or_else(|_| {
                "http://auth.internal/realms/eel/protocol/openid-connect/certs".to_owned()
            })
        });
        let realm = self
            .jwks_realm
            .unwrap_or_else(|| std::env::var("JWKS_REALM").unwrap_or_else(|_| "eel".to_owned()));
        let svc_name = self.otel_service_name.unwrap_or_else(|| {
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "eel-service".to_owned())
        });

        let otel_config = OtelConfig::from_env(svc_name);
        let _otel_guard = init_otel_lenient(&otel_config);

        let jwks = Arc::new(JwksVerifier::new(jwks_url, realm));
        let idempotency = Arc::new(IdempotencyStore::new(self.idempotency_ttl_secs));
        let shedder = Arc::new(CoDelShedder::new(self.codel_config));
        let degradation = Arc::new(self.degradation_flag);

        Ok(ApiServer {
            jwks,
            idempotency,
            shedder,
            degradation,
            bind_addr: self.bind_addr,
        })
    }
}

fn init_otel_lenient(cfg: &OtelConfig) -> Option<eel_observability_rs::OtelGuard> {
    match eel_observability_rs::init_otel(cfg) {
        Ok(g) => Some(g),
        Err(e) => {
            tracing::warn!("OTel init failed (non-fatal in dev): {e}");
            None
        }
    }
}
