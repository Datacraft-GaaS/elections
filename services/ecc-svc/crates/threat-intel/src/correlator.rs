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

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::stix::{StixBundle, StixObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EccFlag {
    pub flag_id: String,
    pub flag_type: String,
    pub value: String,
    pub source: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatch {
    pub flag_id: String,
    pub indicator_id: String,
    pub indicator_name: Option<String>,
    pub pattern: Option<String>,
    pub confidence: f64,
    pub matched_at: String,
}

pub struct Correlator {
    pub bundles: Arc<RwLock<Vec<StixBundle>>>,
}

impl Correlator {
    pub fn new(bundles: Arc<RwLock<Vec<StixBundle>>>) -> Self {
        Self { bundles }
    }

    pub async fn correlate(&self, flag: &EccFlag) -> Vec<CorrelationMatch> {
        let lock = self.bundles.read().await;
        let now = Utc::now().to_rfc3339();
        let mut matches = Vec::new();

        for bundle in lock.iter() {
            for obj in &bundle.objects {
                let indicator = match obj {
                    StixObject::Indicator(i) => i,
                    _ => continue,
                };

                let pattern = match &indicator.pattern {
                    Some(p) => p,
                    None => continue,
                };

                let confidence = if pattern == &flag.value {
                    1.0
                } else if pattern.contains(&flag.value) {
                    0.5
                } else {
                    continue;
                };

                matches.push(CorrelationMatch {
                    flag_id: flag.flag_id.clone(),
                    indicator_id: indicator.common.id.clone(),
                    indicator_name: indicator.common.name.clone(),
                    pattern: Some(pattern.clone()),
                    confidence,
                    matched_at: now.clone(),
                });
            }
        }

        matches
    }

    pub async fn correlate_batch(&self, flags: &[EccFlag]) -> Vec<CorrelationMatch> {
        let mut all = Vec::new();
        for flag in flags {
            let mut results = self.correlate(flag).await;
            all.append(&mut results);
        }
        all
    }
}
