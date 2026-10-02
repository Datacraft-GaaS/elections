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

//! axe-core JSON result parsing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AxeResult {
    #[serde(default)]
    pub violations: Vec<AxeViolation>,
    #[serde(default)]
    pub passes: Vec<serde_json::Value>,
    #[serde(default)]
    pub incomplete: Vec<serde_json::Value>,
    #[serde(default)]
    pub inapplicable: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxeViolation {
    pub id: String,
    #[serde(default)]
    pub impact: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub help: Option<String>,
    #[serde(default)]
    pub help_url: Option<String>,
    #[serde(default)]
    pub nodes: Vec<serde_json::Value>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// AAA-only rule IDs from WCAG 2.2.
const AAA_RULE_IDS: &[&str] = &[
    "color-contrast-enhanced",
    "focus-visible-enhanced",
    "no-autoplay-audio",
    "target-size-enhanced",
    "consistent-identification",
    "error-suggestion",
    "contrast-enhanced",
];

/// Filter violations by the target conformance level.
/// For AA: keep all violations. For AAA: keep all violations (AA already included, AAA adds more).
/// This function filters OUT violations that are strictly AAA-only when testing at AA level.
pub fn filter_violations(violations: &[AxeViolation], level: &str) -> Vec<AxeViolation> {
    if level == "AAA" {
        return violations.to_vec();
    }
    // For AA level, exclude violations that are strictly AAA-only
    violations
        .iter()
        .filter(|v| !AAA_RULE_IDS.contains(&v.id.as_str()))
        .cloned()
        .collect()
}

pub fn parse(json: &str) -> anyhow::Result<AxeResult> {
    let result: AxeResult = serde_json::from_str(json)?;
    Ok(result)
}
