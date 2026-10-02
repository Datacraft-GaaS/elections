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

//! Ensemble<M>: runs N models in parallel (via Rayon) and enforces K-of-N agreement.

use std::collections::HashMap;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::EnsembleError;
use crate::policy::{DisagreementAction, EnsemblePolicy};

/// A model prediction: a label (e.g. "deepfake", "genuine") and a confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Categorical label produced by the model.
    pub label: String,
    /// Confidence in [0.0, 1.0].
    pub confidence: f64,
}

/// Trait for a pluggable AI model.
pub trait Model: Send + Sync {
    /// Human-readable model identifier.
    fn name(&self) -> &str;
    /// Run inference on the serialised input and return a Prediction.
    fn predict(&self, input: &serde_json::Value) -> Result<Prediction, EnsembleError>;
}

/// Voting result from the ensemble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleResult {
    /// The winning label (plurality, subject to K quorum).
    pub label: String,
    /// Average confidence of models that voted for the winning label.
    pub mean_confidence: f64,
    /// Number of models that agreed on the winning label.
    pub agree_count: usize,
    /// Total number of models polled.
    pub total_count: usize,
    /// Per-model votes for audit logging.
    pub votes: Vec<(String, Prediction)>,
}

/// K-of-N ensemble runner.
pub struct Ensemble<M: Model> {
    models: Vec<M>,
    policy: EnsemblePolicy,
}

impl<M: Model> Ensemble<M> {
    pub fn new(models: Vec<M>, policy: EnsemblePolicy) -> Result<Self, EnsembleError> {
        if models.len() != policy.n {
            return Err(EnsembleError::PolicyInvalid {
                k: policy.k,
                n: policy.n,
            });
        }
        Ok(Self { models, policy })
    }

    /// Dispatch all N models in parallel, collect votes, enforce K quorum.
    pub fn predict(&self, input: &serde_json::Value) -> Result<EnsembleResult, EnsembleError> {
        // Parallel predict via Rayon.
        let votes: Vec<(String, Result<Prediction, EnsembleError>)> = self
            .models
            .par_iter()
            .map(|m| (m.name().to_owned(), m.predict(input)))
            .collect();

        // Gather successful predictions.
        let mut successful: Vec<(String, Prediction)> = Vec::new();
        for (name, res) in votes {
            match res {
                Ok(p) => successful.push((name, p)),
                Err(e) => warn!(model = %name, error = %e, "model failed; excluded from vote"),
            }
        }

        // Count votes by label.
        let mut tally: HashMap<&str, (usize, f64)> = HashMap::new();
        for (_, pred) in &successful {
            let e = tally.entry(&pred.label).or_insert((0, 0.0));
            e.0 += 1;
            e.1 += pred.confidence;
        }

        // Find plurality winner.
        let (winning_label, (agree_count, conf_sum)) = tally
            .into_iter()
            .max_by_key(|(_, (count, _))| *count)
            .unwrap_or(("unknown", (0, 0.0)));

        let mean_confidence = if agree_count > 0 {
            conf_sum / agree_count as f64
        } else {
            0.0
        };

        info!(
            label = winning_label,
            agree = agree_count,
            required = self.policy.k,
            total = self.policy.n,
            "ensemble vote result"
        );

        if agree_count < self.policy.k {
            return match self.policy.disagreement_action {
                DisagreementAction::Reject | DisagreementAction::Escalate => {
                    Err(EnsembleError::Disagreement {
                        agreed: agree_count,
                        required: self.policy.k,
                        label: winning_label.to_owned(),
                        action: self.policy.disagreement_action,
                    })
                }
                DisagreementAction::LogOnly => {
                    warn!(
                        agreed = agree_count,
                        required = self.policy.k,
                        "log_only: ensemble below quorum, accepting plurality"
                    );
                    Ok(EnsembleResult {
                        label: winning_label.to_owned(),
                        mean_confidence,
                        agree_count,
                        total_count: successful.len(),
                        votes: successful,
                    })
                }
            };
        }

        Ok(EnsembleResult {
            label: winning_label.to_owned(),
            mean_confidence,
            agree_count,
            total_count: successful.len(),
            votes: successful,
        })
    }
}
