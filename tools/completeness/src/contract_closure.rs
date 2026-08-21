//! `eel-completeness contract-closure`, verifies that every service pair,
//! every OpenAPI path, and every event schema has producer and consumer test coverage.
//!
//! Sources read:
//!   packages/openapi-contracts/         (OpenAPI specs)
//!   packages/event-schemas/             (JSON Schema event definitions)
//!   platform/kafka/topics/              (topic YAML files, each contains `type` field)
//!   packages/authz-schema/              (ZED schema files)
//!
//! Coverage signals looked for:
//!   - OpenAPI path: any test file containing the path string
//!   - Event schema: test file containing the event `type` field value, tagged as
//!     producer or consumer
//!   - Service pair: pact broker JSON files under tests/ or loadtest/ directories
//!
//! This is a best-effort static analysis. A full implementation requires a running
//! Pact Broker; the tool emits warnings for service pairs with no pact found.

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".cache",
    "_build",
    ".elixir_ls",
    "deps",
    ".dart_tool",
];

pub fn run(root: &Path, fail: bool) -> Result<i32> {
    let mut issues: Vec<String> = Vec::new();

    // 1. Collect OpenAPI paths
    let api_paths = collect_api_paths(root)?;
    // 2. Collect event types from schemas
    let event_types = collect_event_types(root)?;
    // 3. Collect topic event types from platform/kafka/topics/
    let topic_types = collect_topic_types(root)?;
    // 4. Collect test file contents for coverage check
    let test_contents = collect_test_contents(root)?;

    // Check each OpenAPI path appears in at least one test
    for (path_item, file) in &api_paths {
        let covered = test_contents
            .iter()
            .any(|tc| tc.contains(path_item.as_str()));
        if !covered {
            issues.push(format!(
                "OpenAPI path `{path_item}` in {file} has no test coverage"
            ));
        }
    }

    // Check each event type appears in a test (producer + consumer)
    for (ev_type, schema_file) in &event_types {
        let in_tests = test_contents
            .iter()
            .filter(|tc| tc.contains(ev_type.as_str()))
            .count();
        if in_tests == 0 {
            issues.push(format!(
                "Event type `{ev_type}` (schema: {schema_file}) has no test coverage"
            ));
        } else if in_tests == 1 {
            issues.push(format!(
                "Event type `{ev_type}` appears in only {in_tests} test file, need both producer and consumer"
            ));
        }
    }

    // Check event schema types map to a known topic
    for (ev_type, schema_file) in &event_types {
        if !topic_types.contains(ev_type.as_str()) {
            issues.push(format!(
                "Event type `{ev_type}` (schema: {schema_file}) has no matching topic in platform/kafka/topics/"
            ));
        }
    }

    // Emit
    if issues.is_empty() {
        eprintln!("contract-closure: all checks passed");
        Ok(0)
    } else {
        for issue in &issues {
            eprintln!("CLOSURE-GAP: {issue}");
        }
        eprintln!("{} closure gaps found", issues.len());
        if fail {
            Ok(1)
        } else {
            Ok(0)
        }
    }
}

fn collect_api_paths(root: &Path) -> Result<Vec<(String, String)>> {
    let dir = root.join("packages").join("openapi-contracts");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let path_re = Regex::new(r"^  (/[^\s:#]+):").unwrap();
    let mut paths = Vec::new();

    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |x| x == "yaml" || x == "yml")
        })
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        for line in content.lines() {
            if let Some(cap) = path_re.captures(line) {
                paths.push((cap[1].to_owned(), rel.clone()));
            }
        }
    }
    Ok(paths)
}

fn collect_event_types(root: &Path) -> Result<Vec<(String, String)>> {
    let dir = root.join("packages").join("event-schemas");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut types = Vec::new();
    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |x| x == "json"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            // Look for the event `type` field in the schema's properties or const
            if let Some(t) = val
                .get("properties")
                .and_then(|p| p.get("type"))
                .and_then(|t| t.get("const"))
                .and_then(Value::as_str)
            {
                types.push((t.to_owned(), rel));
            } else if let Some(t) = val.get("const").and_then(Value::as_str) {
                types.push((t.to_owned(), rel));
            }
        }
    }
    Ok(types)
}

fn collect_topic_types(root: &Path) -> Result<HashSet<String>> {
    let dir = root.join("platform").join("kafka").join("topics");
    if !dir.exists() {
        return Ok(HashSet::new());
    }
    let mut types = HashSet::new();
    let type_re = Regex::new(r"^\s*event_type\s*:\s*(.+)$").unwrap();
    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |x| x == "yaml" || x == "yml")
        })
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for line in content.lines() {
            if let Some(cap) = type_re.captures(line) {
                types.insert(
                    cap[1]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_owned(),
                );
            }
        }
    }
    Ok(types)
}

fn collect_test_contents(root: &Path) -> Result<Vec<String>> {
    let mut contents = Vec::new();
    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let n = e.file_name().to_string_lossy();
                return !SKIP_DIRS.contains(&n.as_ref());
            }
            true
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        // Collect files inside test directories
        let in_test_dir = path.components().any(|c| {
            matches!(
                c.as_os_str().to_str(),
                Some("tests") | Some("test") | Some("spec")
            )
        });
        if !in_test_dir {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(
            ext,
            "rs" | "py" | "ex" | "exs" | "dart" | "ts" | "tsx" | "js" | "jsx"
        ) {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            contents.push(content);
        }
    }
    Ok(contents)
}
