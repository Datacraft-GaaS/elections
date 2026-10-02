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

use eel_ecc_threat_intel::{
    correlator::{Correlator, EccFlag},
    dashboard_export::build_grafana_panel,
    feed::{FeedManager, TaxiiEndpoint},
    stix::{parse_bundle, parse_object, StixBundle, StixObject},
};
use tokio::sync::RwLock;

// ---- helpers ----

fn indicator_json(id: &str, pattern: &str) -> String {
    format!(
        r#"{{
            "type": "indicator",
            "id": "{id}",
            "spec_version": "2.1",
            "created": "2024-01-01T00:00:00Z",
            "modified": "2024-01-01T00:00:00Z",
            "name": "Test Indicator",
            "pattern": "{pattern}",
            "pattern_type": "stix",
            "valid_from": "2024-01-01T00:00:00Z"
        }}"#
    )
}

fn make_bundle_with(objects_json: &str) -> String {
    format!(
        r#"{{
            "type": "bundle",
            "id": "bundle--00000000-0000-0000-0000-000000000000",
            "objects": [{objects_json}]
        }}"#
    )
}

fn flag(value: &str) -> EccFlag {
    EccFlag {
        flag_id: "flag-1".into(),
        flag_type: "ip".into(),
        value: value.into(),
        source: "test".into(),
        timestamp: "2024-01-01T00:00:00Z".into(),
    }
}

async fn correlator_with(bundles: Vec<StixBundle>) -> Correlator {
    let arc = Arc::new(RwLock::new(bundles));
    Correlator::new(arc)
}

// ---- stix parse tests ----

#[test]
fn stix_parse_indicator_roundtrip() {
    let json = indicator_json("indicator--abc", "192.0.2.1");
    let obj = parse_object(&json).unwrap();
    match &obj {
        StixObject::Indicator(i) => {
            assert_eq!(i.common.id, "indicator--abc");
            assert_eq!(i.pattern.as_deref(), Some("192.0.2.1"));
        }
        _ => panic!("expected Indicator"),
    }
    let re = serde_json::to_string(&obj).unwrap();
    assert!(re.contains("indicator--abc"));
    assert!(re.contains("192.0.2.1"));
}

#[test]
fn stix_parse_threat_actor() {
    let json = r#"{
        "type": "threat-actor",
        "id": "threat-actor--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "APT999",
        "aliases": ["Group99", "UNC999"],
        "threat_actor_types": ["nation-state"],
        "sophistication": "advanced"
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::ThreatActor(ta) => {
            assert_eq!(ta.common.name.as_deref(), Some("APT999"));
            assert_eq!(ta.aliases.unwrap().len(), 2);
        }
        _ => panic!("expected ThreatActor"),
    }
}

#[test]
fn stix_parse_campaign() {
    let json = r#"{
        "type": "campaign",
        "id": "campaign--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "Operation Ballot",
        "objective": "voter suppression"
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::Campaign(c) => {
            assert_eq!(c.objective.as_deref(), Some("voter suppression"));
        }
        _ => panic!("expected Campaign"),
    }
}

#[test]
fn stix_parse_malware() {
    let json = r#"{
        "type": "malware",
        "id": "malware--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "VoteWiper",
        "malware_types": ["wiper"],
        "is_family": false
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::Malware(m) => {
            assert_eq!(m.is_family, Some(false));
            assert!(m.malware_types.unwrap().contains(&"wiper".to_string()));
        }
        _ => panic!("expected Malware"),
    }
}

#[test]
fn stix_parse_attack_pattern() {
    let json = r#"{
        "type": "attack-pattern",
        "id": "attack-pattern--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "Phishing",
        "kill_chain_phases": [{"kill_chain_name":"mitre-attack","phase_name":"initial-access"}]
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::AttackPattern(ap) => {
            assert!(ap.kill_chain_phases.is_some());
        }
        _ => panic!("expected AttackPattern"),
    }
}

#[test]
fn stix_parse_tool() {
    let json = r#"{
        "type": "tool",
        "id": "tool--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "Mimikatz",
        "tool_types": ["credential-exploitation"],
        "tool_version": "2.2.1"
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::Tool(t) => {
            assert_eq!(t.tool_version.as_deref(), Some("2.2.1"));
        }
        _ => panic!("expected Tool"),
    }
}

#[test]
fn stix_parse_vulnerability() {
    let json = r#"{
        "type": "vulnerability",
        "id": "vulnerability--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "CVE-2024-12345",
        "external_references": [{"source_name":"cve","external_id":"CVE-2024-12345"}]
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::Vulnerability(v) => {
            assert!(v.external_references.is_some());
        }
        _ => panic!("expected Vulnerability"),
    }
}

#[test]
fn stix_parse_intrusion_set() {
    let json = r#"{
        "type": "intrusion-set",
        "id": "intrusion-set--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "name": "Fancy Bear",
        "aliases": ["APT28"],
        "goals": ["election interference"]
    }"#;
    let obj = parse_object(json).unwrap();
    match obj {
        StixObject::IntrusionSet(is) => {
            assert_eq!(is.aliases.unwrap(), vec!["APT28"]);
            assert_eq!(is.goals.unwrap(), vec!["election interference"]);
        }
        _ => panic!("expected IntrusionSet"),
    }
}

#[test]
fn stix_parse_bundle_with_mixed_objects() {
    let bundle_json = r#"{
        "type": "bundle",
        "id": "bundle--mixed",
        "objects": [
            {
                "type": "indicator",
                "id": "indicator--a1",
                "spec_version": "2.1",
                "created": "2024-01-01T00:00:00Z",
                "modified": "2024-01-01T00:00:00Z",
                "pattern": "10.0.0.1",
                "pattern_type": "stix",
                "valid_from": "2024-01-01T00:00:00Z"
            },
            {
                "type": "malware",
                "id": "malware--b2",
                "spec_version": "2.1",
                "created": "2024-01-01T00:00:00Z",
                "modified": "2024-01-01T00:00:00Z",
                "name": "TestMalware"
            },
            {
                "type": "campaign",
                "id": "campaign--c3",
                "spec_version": "2.1",
                "created": "2024-01-01T00:00:00Z",
                "modified": "2024-01-01T00:00:00Z",
                "name": "TestCampaign"
            }
        ]
    }"#;
    let bundle = parse_bundle(bundle_json).unwrap();
    assert_eq!(bundle.objects.len(), 3);
    assert!(matches!(bundle.objects[0], StixObject::Indicator(_)));
    assert!(matches!(bundle.objects[1], StixObject::Malware(_)));
    assert!(matches!(bundle.objects[2], StixObject::Campaign(_)));
}

#[test]
fn stix_unknown_type_handled() {
    let json = r#"{
        "type": "x-custom-object",
        "id": "x-custom-object--1",
        "spec_version": "2.1",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "custom_field": "custom_value"
    }"#;
    let obj = parse_object(json).unwrap();
    assert!(matches!(obj, StixObject::Unknown(_)));
}

// ---- correlator tests ----

async fn setup_correlator(pattern: &str) -> Correlator {
    let ind_json = indicator_json("indicator--test", pattern);
    let bundle_json = make_bundle_with(&ind_json);
    let bundle = parse_bundle(&bundle_json).unwrap();
    correlator_with(vec![bundle]).await
}

#[tokio::test]
async fn correlator_flags_known_indicator() {
    let c = setup_correlator("192.0.2.1").await;
    let f = flag("192.0.2.1");
    let matches = c.correlate(&f).await;
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].confidence, 1.0);
}

#[tokio::test]
async fn correlator_unknown_not_matched() {
    let c = setup_correlator("192.0.2.1").await;
    let f = flag("10.0.0.1");
    let matches = c.correlate(&f).await;
    assert!(matches.is_empty());
}

#[tokio::test]
async fn correlator_batch_multiple_flags() {
    let c = setup_correlator("192.0.2.1").await;
    let flags = vec![
        EccFlag {
            flag_id: "f1".into(),
            flag_type: "ip".into(),
            value: "192.0.2.1".into(),
            source: "test".into(),
            timestamp: "2024-01-01T00:00:00Z".into(),
        },
        EccFlag {
            flag_id: "f2".into(),
            flag_type: "ip".into(),
            value: "10.0.0.1".into(),
            source: "test".into(),
            timestamp: "2024-01-01T00:00:00Z".into(),
        },
        EccFlag {
            flag_id: "f3".into(),
            flag_type: "ip".into(),
            value: "172.16.0.1".into(),
            source: "test".into(),
            timestamp: "2024-01-01T00:00:00Z".into(),
        },
    ];
    let matches = c.correlate_batch(&flags).await;
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].flag_id, "f1");
}

// ---- feed manager tests ----

#[test]
fn feed_manager_default_endpoints_count() {
    let eps = FeedManager::default_endpoints();
    assert_eq!(eps.len(), 3);
}

#[tokio::test]
async fn feed_manager_mock_taxii_fetch() {
    use httpmock::prelude::*;

    let bundle_json = r#"{
        "type": "bundle",
        "id": "bundle--mock",
        "objects": [
            {
                "type": "indicator",
                "id": "indicator--mock-1",
                "spec_version": "2.1",
                "created": "2024-01-01T00:00:00Z",
                "modified": "2024-01-01T00:00:00Z",
                "pattern": "198.51.100.1",
                "pattern_type": "stix",
                "valid_from": "2024-01-01T00:00:00Z"
            }
        ]
    }"#;

    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/api/stix/test-collection/objects/");
        then.status(200)
            .header("Content-Type", "application/stix+json;version=2.1")
            .body(bundle_json);
    });

    let endpoint = TaxiiEndpoint {
        url: server.base_url(),
        collection_id: "test-collection".into(),
        api_root: "stix".into(),
        label: "Mock Feed".into(),
    };

    let manager = FeedManager::new(vec![endpoint]);
    manager.fetch_all().await.unwrap();

    let bundles = manager.bundles().await;
    assert_eq!(bundles.len(), 1);
    assert_eq!(bundles[0].objects.len(), 1);
}

// ---- dashboard export tests ----

#[test]
fn grafana_panel_build() {
    use eel_ecc_threat_intel::correlator::CorrelationMatch;

    let matches = vec![
        CorrelationMatch {
            flag_id: "f1".into(),
            indicator_id: "indicator--a".into(),
            indicator_name: Some("Indicator A".into()),
            pattern: Some("192.0.2.1".into()),
            confidence: 1.0,
            matched_at: "2024-01-01T00:00:00Z".into(),
        },
        CorrelationMatch {
            flag_id: "f2".into(),
            indicator_id: "indicator--b".into(),
            indicator_name: None,
            pattern: Some("10.1.1.1".into()),
            confidence: 0.5,
            matched_at: "2024-01-01T00:00:00Z".into(),
        },
    ];

    let panel = build_grafana_panel(&matches);
    assert_eq!(panel.title, "ECC Threat Correlation Matches");
    assert_eq!(panel.type_, "table");
    assert_eq!(panel.rows.len(), 2);
    assert!(!panel.columns.is_empty());
}
