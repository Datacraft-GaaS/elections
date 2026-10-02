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

use eel_wcag_audit_lib::{audit, axe, manifest, report};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

#[test]
fn parse_axe_pass_fixture_has_zero_violations() {
    let path = fixtures_dir().join("axe-pass.json");
    let json = std::fs::read_to_string(&path).expect("reading axe-pass.json");
    let result = axe::parse(&json).expect("parsing axe result");
    assert_eq!(
        result.violations.len(),
        0,
        "pass fixture should have 0 violations"
    );
    assert_eq!(
        result.passes.len(),
        20,
        "pass fixture should have 20 passes"
    );
}

#[test]
fn parse_axe_fail_fixture_has_two_violations() {
    let path = fixtures_dir().join("axe-fail.json");
    let json = std::fs::read_to_string(&path).expect("reading axe-fail.json");
    let result = axe::parse(&json).expect("parsing axe result");
    assert_eq!(
        result.violations.len(),
        2,
        "fail fixture should have 2 violations"
    );
    let ids: Vec<&str> = result.violations.iter().map(|v| v.id.as_str()).collect();
    assert!(
        ids.contains(&"color-contrast"),
        "missing color-contrast violation"
    );
    assert!(ids.contains(&"image-alt"), "missing image-alt violation");
    assert_eq!(
        result.passes.len(),
        18,
        "fail fixture should have 18 passes"
    );
}

#[test]
fn report_json_from_pass_fixture_is_valid() {
    let fixtures = fixtures_dir();
    let entry = manifest::ManifestEntry {
        url: "http://localhost:3000".to_string(),
        name: "public-portal".to_string(),
        module: "M11".to_string(),
        level: "AA".to_string(),
    };
    let result = audit::audit_entry(&entry, Some(&fixtures)).expect("audit_entry with stub");
    assert_eq!(result.status, "pass");
    assert_eq!(result.violations.len(), 0);

    let rep = report::Report::from_results(vec![result]);
    assert!(!rep.has_violations());
    assert_eq!(rep.summary.total_urls, 1);
    assert_eq!(rep.summary.passing, 1);
    assert_eq!(rep.summary.failing, 0);

    let json_str = report::render_json(&rep).expect("render JSON");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("output is valid JSON");
    assert_eq!(parsed["summary"]["total_urls"], 1);
    assert_eq!(parsed["summary"]["passing"], 1);
    assert_eq!(parsed["summary"]["violations_total"], 0);
}

#[test]
fn report_from_fail_fixture_shows_violations() {
    let fixtures = fixtures_dir();
    // Write a slug-named stub for the URL so audit_entry picks it up
    let tmp = tempfile::tempdir().unwrap();
    let slug = manifest::url_slug("http://localhost:3001");
    let stub_path = tmp.path().join(format!("{slug}.json"));
    let fail_json = std::fs::read_to_string(fixtures.join("axe-fail.json")).unwrap();
    std::fs::write(&stub_path, &fail_json).unwrap();

    let entry = manifest::ManifestEntry {
        url: "http://localhost:3001".to_string(),
        name: "test-page".to_string(),
        module: "M99".to_string(),
        level: "AA".to_string(),
    };
    let result = audit::audit_entry(&entry, Some(tmp.path())).expect("audit with fail stub");
    assert_eq!(result.status, "fail");
    assert_eq!(result.violations.len(), 2);

    let rep = report::Report::from_results(vec![result]);
    assert!(rep.has_violations());
    assert_eq!(rep.summary.violations_total, 2);
    assert_eq!(rep.summary.failing, 1);
    assert_eq!(rep.summary.passing, 0);
}

#[test]
fn aaa_filter_keeps_all_violations() {
    let path = fixtures_dir().join("axe-fail.json");
    let json = std::fs::read_to_string(&path).unwrap();
    let result = axe::parse(&json).unwrap();
    let filtered = axe::filter_violations(&result.violations, "AAA");
    assert_eq!(filtered.len(), 2);
}

#[test]
fn aa_filter_excludes_aaa_only_rules() {
    let violations = vec![
        axe::AxeViolation {
            id: "color-contrast".to_string(),
            impact: Some("serious".to_string()),
            description: None,
            help: None,
            help_url: None,
            nodes: vec![],
            tags: vec!["wcag2aa".to_string()],
        },
        axe::AxeViolation {
            id: "color-contrast-enhanced".to_string(),
            impact: Some("serious".to_string()),
            description: None,
            help: None,
            help_url: None,
            nodes: vec![],
            tags: vec!["wcag2aaa".to_string()],
        },
    ];
    let filtered_aa = axe::filter_violations(&violations, "AA");
    assert_eq!(filtered_aa.len(), 1);
    assert_eq!(filtered_aa[0].id, "color-contrast");

    let filtered_aaa = axe::filter_violations(&violations, "AAA");
    assert_eq!(filtered_aaa.len(), 2);
}
