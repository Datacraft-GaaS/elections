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

//! `eel-abis-drift-monitor`: feature-space drift detection and alerting.
//!
//! Computes PSI, KS statistic, Jensen-Shannon divergence, and Wasserstein distance
//! across configurable rolling windows (24h, 7d, 30d) against a baseline snapshot.
#![warn(missing_docs)]

pub mod adaptive;
pub mod alert;
pub mod detector;
pub mod domain;
pub mod error;
pub mod window;

pub use detector::{js_divergence, ks_statistic, psi, wasserstein_distance, DriftDetector};
pub use domain::{DriftAlert, DriftReport, DriftSeverity, DriftWindow, FeatureDistribution};
pub use error::DriftError;
