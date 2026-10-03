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

use eel_resilience_rs::circuit_breaker::CircuitBreaker;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct IprsVerifyRequest {
    pub national_id: String,
    pub full_name: String,
    pub dob: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IprsVerifyResponse {
    pub verified: bool,
    pub message: Option<String>,
}

/// IPRS validation outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IprsOutcome {
    Verified,
    NotVerified(String),
    /// Circuit open or network error - fail closed.
    CircuitOpen,
}

#[derive(Clone)]
pub struct IprsClient {
    base_url: String,
    circuit_breaker: CircuitBreaker,
    http: reqwest::Client,
}

impl IprsClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let cb = CircuitBreaker::new("iprs", 5, Duration::from_secs(60), Duration::from_secs(30));
        Self {
            base_url: base_url.into(),
            circuit_breaker: cb,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Verify national ID via IPRS. Fail-closed: circuit open returns CircuitOpen.
    pub async fn verify(&self, req: IprsVerifyRequest) -> IprsOutcome {
        if self.circuit_breaker.allow().await.is_err() {
            return IprsOutcome::CircuitOpen;
        }
        let url = format!("{}/iprs/verify", self.base_url);
        match self.http.post(&url).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                self.circuit_breaker.record_success().await;
                match resp.json::<IprsVerifyResponse>().await {
                    Ok(body) => {
                        if body.verified {
                            IprsOutcome::Verified
                        } else {
                            IprsOutcome::NotVerified(body.message.unwrap_or_default())
                        }
                    }
                    Err(_) => {
                        self.circuit_breaker.record_failure().await;
                        IprsOutcome::CircuitOpen
                    }
                }
            }
            Ok(_) => {
                self.circuit_breaker.record_failure().await;
                IprsOutcome::CircuitOpen
            }
            Err(_) => {
                self.circuit_breaker.record_failure().await;
                IprsOutcome::CircuitOpen
            }
        }
    }
}
