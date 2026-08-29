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
