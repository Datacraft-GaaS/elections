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

use eel_observability_rs::{
    audit_shadow::audit_shadow_sql, otel::OtelConfig, scrubber::PiiScrubberLayer,
    slo::SloDefinition,
};
use eel_pii_rs::{PIIRestricted, PIISensitive};
use tracing_subscriber::layer::SubscriberExt;

#[test]
fn slo_definition_creates_recording_rule() {
    let def = SloDefinition::new("rts-svc", "availability", 0.999);
    assert_eq!(def.service, "rts-svc");
    assert_eq!(def.indicator, "availability");
    assert!((def.target - 0.999).abs() < f64::EPSILON);
    assert!(!def.recording_rule.is_empty());
    assert!(def.recording_rule.contains("rts-svc"));
}

#[test]
fn slo_macro_produces_definition() {
    use eel_observability_rs::slo;
    let def = slo!(service: "bvr-svc", indicator: "latency_p99", target: 0.95);
    assert_eq!(def.service, "bvr-svc");
    assert_eq!(def.target, 0.95);
}

#[test]
fn audit_shadow_sql_contains_trigger() {
    let sql = audit_shadow_sql("voters");
    assert!(sql.contains("audit.voters_history"));
    assert!(sql.contains("CREATE TRIGGER audit_shadow"));
    assert!(sql.contains("AFTER INSERT OR UPDATE OR DELETE"));
    assert!(sql.contains("changed_by"));
    assert!(sql.contains("request_id"));
}

#[test]
fn pii_restricted_debug_is_redacted() {
    let val = PIIRestricted::new("KE12345678".to_string());
    let debug_str = format!("{val:?}");
    assert!(
        !debug_str.contains("KE12345678"),
        "PIIRestricted leaked in Debug: {debug_str}"
    );
    assert!(
        debug_str.contains("REDACTED"),
        "expected redaction marker: {debug_str}"
    );
}

#[test]
fn pii_sensitive_debug_is_redacted() {
    let val = PIISensitive::new("polling-station-007".to_string());
    let debug_str = format!("{val:?}");
    assert!(
        !debug_str.contains("polling-station-007"),
        "PIISensitive leaked in Debug: {debug_str}"
    );
    assert!(
        debug_str.contains("REDACTED"),
        "expected redaction marker: {debug_str}"
    );
}

#[test]
fn pii_scrubber_layer_is_composable() {
    // Compile-time check: PiiScrubberLayer implements Layer<S> and can be composed.
    let _sub = tracing_subscriber::registry().with(PiiScrubberLayer);
}

#[test]
fn otel_config_from_env() {
    let config = OtelConfig::from_env("test-service");
    assert_eq!(config.service_name, "test-service");
    assert!(!config.traces_endpoint.is_empty());
    assert!(!config.metrics_endpoint.is_empty());
}
