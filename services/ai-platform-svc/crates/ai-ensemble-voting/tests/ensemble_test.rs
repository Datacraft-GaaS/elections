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

//! Tests for K-of-N ensemble voting (HARD-22). 15+ cases.

use eel_ai_ensemble_voting::{
    ensemble::{Ensemble, Model, Prediction},
    error::EnsembleError,
    policy::{DisagreementAction, EnsemblePolicy},
    registry::{DecisionClass, ModelRegistry},
};
use serde_json::json;

/// Stub model: always returns the given label at the given confidence.
struct FixedModel {
    name: String,
    label: String,
    confidence: f64,
}

impl FixedModel {
    fn new(name: &str, label: &str, confidence: f64) -> Self {
        Self {
            name: name.to_owned(),
            label: label.to_owned(),
            confidence,
        }
    }
}

impl Model for FixedModel {
    fn name(&self) -> &str {
        &self.name
    }
    fn predict(&self, _input: &serde_json::Value) -> Result<Prediction, EnsembleError> {
        Ok(Prediction {
            label: self.label.clone(),
            confidence: self.confidence,
        })
    }
}

fn policy(k: usize, n: usize, action: DisagreementAction) -> EnsemblePolicy {
    EnsemblePolicy::new(k, n, action).expect("policy should be valid")
}

fn agree_models(n: usize, label: &str) -> Vec<FixedModel> {
    (0..n)
        .map(|i| FixedModel::new(&format!("m{i}"), label, 0.9))
        .collect()
}

// ---- policy validation ----

#[test]
fn policy_k_greater_than_n_rejected() {
    let err = EnsemblePolicy::new(4, 3, DisagreementAction::Reject).unwrap_err();
    assert!(matches!(err, EnsembleError::PolicyInvalid { k: 4, n: 3 }));
}

#[test]
fn policy_k_equals_n_accepted() {
    EnsemblePolicy::new(3, 3, DisagreementAction::Reject).expect("k==n is valid");
}

#[test]
fn policy_k_zero_accepted() {
    EnsemblePolicy::new(0, 3, DisagreementAction::Reject).expect("k=0 is valid");
}

// ---- K-of-N agreement enforced ----

#[test]
fn all_models_agree_returns_label() {
    let models = agree_models(3, "genuine");
    let p = policy(3, 3, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({})).unwrap();
    assert_eq!(result.label, "genuine");
    assert_eq!(result.agree_count, 3);
}

#[test]
fn k_of_n_majority_succeeds() {
    // 2 say "deepfake", 1 says "genuine"; k=2 -> passes
    let models = vec![
        FixedModel::new("m0", "deepfake", 0.8),
        FixedModel::new("m1", "deepfake", 0.85),
        FixedModel::new("m2", "genuine", 0.6),
    ];
    let p = policy(2, 3, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({})).unwrap();
    assert_eq!(result.label, "deepfake");
    assert_eq!(result.agree_count, 2);
}

#[test]
fn insufficient_agreement_reject_action_errors() {
    // 1 says "deepfake", 1 says "genuine", 1 says "uncertain"; k=2
    let models = vec![
        FixedModel::new("m0", "deepfake", 0.7),
        FixedModel::new("m1", "genuine", 0.6),
        FixedModel::new("m2", "uncertain", 0.5),
    ];
    let p = policy(2, 3, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let err = ensemble.predict(&json!({})).unwrap_err();
    assert!(matches!(err, EnsembleError::Disagreement { .. }));
}

// ---- disagreement_action honoured ----

#[test]
fn disagreement_action_escalate_errors() {
    let models = vec![
        FixedModel::new("m0", "a", 0.5),
        FixedModel::new("m1", "b", 0.5),
    ];
    let p = policy(2, 2, DisagreementAction::Escalate);
    let ensemble = Ensemble::new(models, p).unwrap();
    let err = ensemble.predict(&json!({})).unwrap_err();
    assert!(matches!(
        err,
        EnsembleError::Disagreement {
            action: DisagreementAction::Escalate,
            ..
        }
    ));
}

#[test]
fn disagreement_action_log_only_returns_plurality() {
    let models = vec![
        FixedModel::new("m0", "a", 0.5),
        FixedModel::new("m1", "b", 0.5),
        FixedModel::new("m2", "b", 0.5),
    ];
    // k=3 but only 2 agree -> log_only -> accepts
    let p = policy(3, 3, DisagreementAction::LogOnly);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({})).unwrap();
    assert_eq!(result.label, "b");
}

// ---- parallel dispatch ----

#[test]
fn parallel_dispatch_all_models_queried() {
    let n = 5;
    let models = agree_models(n, "genuine");
    let p = policy(3, n, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({"station": "47-001"})).unwrap();
    assert_eq!(result.total_count, n);
    assert_eq!(result.votes.len(), n);
}

#[test]
fn parallel_dispatch_vote_names_unique() {
    let models: Vec<FixedModel> = (0..4)
        .map(|i| FixedModel::new(&format!("model-{i}"), "genuine", 0.9))
        .collect();
    let p = policy(2, 4, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({})).unwrap();
    let names: std::collections::HashSet<_> = result.votes.iter().map(|(n, _)| n).collect();
    assert_eq!(names.len(), 4);
}

// ---- monotonicity ----

#[test]
fn adding_agreeing_model_never_reduces_agree_count() {
    // Baseline: 2 agree out of 2.
    let base_models = agree_models(2, "genuine");
    let p_base = policy(2, 2, DisagreementAction::Reject);
    let base = Ensemble::new(base_models, p_base)
        .unwrap()
        .predict(&json!({}))
        .unwrap();

    // Extended: 3 agree out of 3.
    let ext_models = agree_models(3, "genuine");
    let p_ext = policy(2, 3, DisagreementAction::Reject);
    let ext = Ensemble::new(ext_models, p_ext)
        .unwrap()
        .predict(&json!({}))
        .unwrap();

    assert!(ext.agree_count >= base.agree_count);
}

#[test]
fn mean_confidence_within_unit_interval() {
    let models = agree_models(3, "deepfake");
    let p = policy(2, 3, DisagreementAction::Reject);
    let ensemble = Ensemble::new(models, p).unwrap();
    let result = ensemble.predict(&json!({})).unwrap();
    assert!((0.0..=1.0).contains(&result.mean_confidence));
}

// ---- registry ----

#[test]
fn registry_registers_and_retrieves_policy() {
    let mut reg = ModelRegistry::new();
    let p = policy(2, 3, DisagreementAction::Reject);
    reg.register(DecisionClass::Deepfake, p);
    assert!(reg.get(&DecisionClass::Deepfake).is_some());
}

#[test]
fn registry_missing_class_returns_none() {
    let reg = ModelRegistry::new();
    assert!(reg.get(&DecisionClass::Anomaly).is_none());
}

#[test]
fn registry_remove_clears_entry() {
    let mut reg = ModelRegistry::new();
    let p = policy(1, 2, DisagreementAction::LogOnly);
    reg.register(DecisionClass::Cib, p);
    reg.remove(&DecisionClass::Cib);
    assert!(reg.get(&DecisionClass::Cib).is_none());
}

#[test]
fn registry_custom_class_round_trips() {
    let mut reg = ModelRegistry::new();
    let p = policy(1, 1, DisagreementAction::Reject);
    reg.register(DecisionClass::Custom("ballot_quality".to_owned()), p);
    assert!(reg
        .get(&DecisionClass::Custom("ballot_quality".to_owned()))
        .is_some());
}
