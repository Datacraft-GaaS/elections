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

//! eel-ai-platform-telemetry-analytics
//!
//! Mobile-telemetry analytics and anomaly detection for the eel platform.
//! Implements the nine detectors specified in mobile-observability spec §7.
#![warn(missing_docs)]

pub mod api;
pub mod backtest;
pub mod config;
pub mod detector;
pub mod dispatch;
pub mod metrics;
pub mod pipeline;
pub mod rule_registry;

pub mod detectors {
    pub mod battery_starvation;
    pub mod coordinated;
    pub mod drop;
    pub mod geo_fence;
    pub mod login_time;
    pub mod read_heavy;
    pub mod rfid_absence;
    pub mod session_hijack;
    pub mod ueba;
}

pub use detector::{Alert, AlertSeverity, Detector, SignalStream};
pub use pipeline::AnalyticsPipeline;
pub use rule_registry::RuleRegistry;
