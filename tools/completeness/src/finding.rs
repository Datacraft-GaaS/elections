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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: String,
    pub message: String,
    pub severity: Severity,
    /// Whether this finding appears in the allowlist (not suppressed if strict mode)
    pub allowlisted: bool,
    /// ADR reference from the allowlist entry, if present
    pub adr: Option<String>,
    /// Allowlist expiry date (ISO-8601), if present
    pub expires: Option<String>,
}

impl Finding {
    pub fn new(
        file: impl Into<String>,
        line: usize,
        column: usize,
        kind: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
    ) -> Self {
        let file = file.into();
        let kind = kind.into();
        let id = format!("{file}:{line}:{kind}");
        Self {
            id,
            file,
            line,
            column,
            kind,
            message: message.into(),
            severity,
            allowlisted: false,
            adr: None,
            expires: None,
        }
    }
}
