//! ModelRegistry: maps decision classes to ensemble policies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::policy::EnsemblePolicy;

/// Decision classes for which ensembles are configured.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionClass {
    /// Deepfake / synthetic media detection.
    Deepfake,
    /// Anomaly / outlier detection.
    Anomaly,
    /// Coordinated inauthentic behaviour detection.
    Cib,
    /// Audio / video transcription quality.
    Transcription,
    /// Custom / extension class.
    Custom(String),
}

/// Registry of ensemble policies keyed by decision class.
#[derive(Debug, Default)]
pub struct ModelRegistry {
    policies: HashMap<String, EnsemblePolicy>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(class: &DecisionClass) -> String {
        match class {
            DecisionClass::Deepfake => "deepfake".to_owned(),
            DecisionClass::Anomaly => "anomaly".to_owned(),
            DecisionClass::Cib => "cib".to_owned(),
            DecisionClass::Transcription => "transcription".to_owned(),
            DecisionClass::Custom(s) => s.clone(),
        }
    }

    pub fn register(&mut self, class: DecisionClass, policy: EnsemblePolicy) {
        self.policies.insert(Self::key(&class), policy);
    }

    pub fn get(&self, class: &DecisionClass) -> Option<&EnsemblePolicy> {
        self.policies.get(&Self::key(class))
    }

    pub fn remove(&mut self, class: &DecisionClass) -> Option<EnsemblePolicy> {
        self.policies.remove(&Self::key(class))
    }

    pub fn classes(&self) -> Vec<String> {
        self.policies.keys().cloned().collect()
    }
}
