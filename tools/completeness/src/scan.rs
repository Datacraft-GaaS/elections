//! `eel-completeness scan` — regex-based stub/skip finder.
//!
//! Patterns (from docs/plans/11-completeness-audit.md §9.1):
//!   Rust:   todo!(), unimplemented!(), #[ignore], empty match arms: _ => Unsupported
//!   Python: TODO/FIXME/XXX comments, NotImplementedError, pass-only bodies, pytest.mark.skip
//!   Elixir: TODO/FIXME/XXX, @tag :skip, UnimplementedError
//!   Dart:   TODO/FIXME/XXX, it.skip/test.skip/xit
//!   TS/JS:  TODO/FIXME/XXX, it.skip/xit/test.skip, 501 handlers
//!   OpenAPI: paths without x-handler-bound annotation
//!   Kotlin: TODO(), NotImplementedError
//!
//! Note: tree-sitter grammars are optional (requires Nix-installed grammars).
//! This implementation uses regex, which covers the patterns reliably.

use crate::allowlist::Allowlist;
use crate::finding::{Finding, Severity};
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

/// Skip these directories entirely.
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

struct Pattern {
    kind: &'static str,
    re: Regex,
    severity: Severity,
    message: &'static str,
    extensions: &'static [&'static str],
}

fn build_patterns() -> Vec<Pattern> {
    vec![
        // Rust: todo!() macro
        Pattern {
            kind: "todo_macro",
            re: Regex::new(r"\btodo!\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "todo!() — unimplemented code path",
            extensions: &["rs"],
        },
        // Rust: unimplemented!() macro
        Pattern {
            kind: "unimplemented_macro",
            re: Regex::new(r"\bunimplemented!\s*\(").unwrap(),
            severity: Severity::Error,
            message: "unimplemented!() — this will panic at runtime",
            extensions: &["rs"],
        },
        // Rust: #[ignore] on a test
        Pattern {
            kind: "ignored_test",
            re: Regex::new(r"#\[ignore\b").unwrap(),
            severity: Severity::Warning,
            message: "#[ignore] — test is skipped",
            extensions: &["rs"],
        },
        // Rust: empty match arm returning Unsupported/unimplemented
        Pattern {
            kind: "empty_match_unsupported",
            re: Regex::new(r"=>\s*(?:Err\s*\(\s*)?(?:Status::)?[Uu]nsupported\b|=>\s*unimplemented!\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "empty/unsupported match arm",
            extensions: &["rs"],
        },
        // Python: pass-only body (heuristic: bare pass on its own line)
        Pattern {
            kind: "pass_only_body",
            re: Regex::new(r"^\s+pass\s*$").unwrap(),
            severity: Severity::Info,
            message: "bare `pass` — placeholder body",
            extensions: &["py"],
        },
        // Python: NotImplementedError
        Pattern {
            kind: "not_implemented_error",
            re: Regex::new(r"\bNotImplementedError\b").unwrap(),
            severity: Severity::Warning,
            message: "NotImplementedError — stub function",
            extensions: &["py"],
        },
        // Python: pytest.mark.skip
        Pattern {
            kind: "pytest_skip",
            re: Regex::new(r"pytest\.mark\.skip\b").unwrap(),
            severity: Severity::Warning,
            message: "pytest.mark.skip — test is skipped",
            extensions: &["py"],
        },
        // Elixir: @tag :skip
        Pattern {
            kind: "exunit_tag_skip",
            re: Regex::new(r"@tag\s+:skip\b").unwrap(),
            severity: Severity::Warning,
            message: "@tag :skip — ExUnit test is skipped",
            extensions: &["ex", "exs"],
        },
        // Elixir: UnimplementedError (not stdlib but common in generated stubs)
        Pattern {
            kind: "elixir_unimplemented",
            re: Regex::new(r"\bUnimplementedError\b").unwrap(),
            severity: Severity::Error,
            message: "UnimplementedError — Elixir stub",
            extensions: &["ex", "exs"],
        },
        // Dart/Flutter: it.skip, xit, test.skip
        Pattern {
            kind: "dart_skip",
            re: Regex::new(r"\b(?:it\.skip|xit|test\.skip)\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "skipped Dart/Flutter test",
            extensions: &["dart"],
        },
        // JS/TS: it.skip, xit, test.skip, describe.skip
        Pattern {
            kind: "js_skip",
            re: Regex::new(r"\b(?:it\.skip|xit|xdescribe|describe\.skip|test\.skip)\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "skipped JS/TS test",
            extensions: &["ts", "tsx", "js", "jsx", "mts", "cts"],
        },
        // Generic: TODO/FIXME/XXX comment in any source file
        Pattern {
            kind: "todo_comment",
            re: Regex::new(r"(?://|#|<!--|--|%%)\s*(?:TODO|FIXME|XXX)\b").unwrap(),
            severity: Severity::Info,
            message: "TODO/FIXME/XXX comment",
            extensions: &[
                "rs", "py", "ex", "exs", "dart", "ts", "tsx", "js", "jsx",
                "mts", "cts", "kt", "java", "go", "yaml", "yml",
            ],
        },
        // HTTP 501 handler (Not Implemented)
        Pattern {
            kind: "http_501",
            re: Regex::new(r"\b501\b.*[Nn]ot\s+[Ii]mplemented|StatusCode::NOT_IMPLEMENTED|http\.StatusNotImplemented").unwrap(),
            severity: Severity::Error,
            message: "HTTP 501 Not Implemented response — stub endpoint",
            extensions: &["rs", "py", "ts", "js", "go"],
        },
        // Kotlin: TODO() / NotImplementedError
        Pattern {
            kind: "kotlin_stub",
            re: Regex::new(r"\bTODO\s*\(\s*\)|\bNotImplementedError\b").unwrap(),
            severity: Severity::Warning,
            message: "Kotlin TODO()/NotImplementedError — stub",
            extensions: &["kt"],
        },
        // default-off feature flag without enabling test
        // (heuristic: feature flag set to false with no test referencing the same flag name nearby)
        Pattern {
            kind: "default_off_flag",
            re: Regex::new(r#"(?:default\s*:\s*false|enabled\s*:\s*false)\s*(?:#[^\n]*)?\n"#).unwrap(),
            severity: Severity::Info,
            message: "default-off feature flag — verify an enabling test exists",
            extensions: &["yaml", "yml", "toml"],
        },
    ]
}

fn extension(path: &Path) -> Option<&str> {
    path.extension()?.to_str()
}

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

pub fn run(root: &Path, allowlist: &Allowlist) -> Result<Vec<Finding>> {
    let patterns = build_patterns();
    let mut findings: Vec<Finding> = Vec::new();

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if e.file_type().is_dir() && should_skip_dir(&name) {
                return false;
            }
            true
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = match extension(path) {
            Some(e) => e,
            None => continue,
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        // Only read files whose extension is covered by at least one pattern.
        let applicable: Vec<&Pattern> = patterns
            .iter()
            .filter(|p| p.extensions.contains(&ext))
            .collect();
        if applicable.is_empty() {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // binary file or permission error — skip
        };

        for (lineno, line) in content.lines().enumerate() {
            let lineno = lineno + 1;
            for pat in &applicable {
                if let Some(m) = pat.re.find(line) {
                    let mut f = Finding::new(
                        &rel,
                        lineno,
                        m.start() + 1,
                        pat.kind,
                        pat.message,
                        pat.severity.clone(),
                    );
                    if allowlist.is_active(&f.id) {
                        if let Some(entry) = allowlist.get(&f.id) {
                            f.allowlisted = true;
                            f.adr = entry.adr.clone();
                            f.expires = entry.expires.clone();
                        }
                    }
                    findings.push(f);
                }
            }
        }
    }

    // OpenAPI paths without x-handler-bound (scan *.yaml in packages/openapi-contracts)
    findings.extend(scan_openapi_unbound(root, allowlist)?);

    Ok(findings)
}

/// Scan OpenAPI YAML files for paths lacking an x-handler-bound extension.
fn scan_openapi_unbound(root: &Path, allowlist: &Allowlist) -> Result<Vec<Finding>> {
    let contracts_dir = root.join("packages").join("openapi-contracts");
    if !contracts_dir.exists() {
        return Ok(vec![]);
    }
    let mut findings = Vec::new();
    // Very lightweight heuristic: if a path item has no `x-handler-bound` key,
    // flag it. A proper implementation would parse the YAML AST.
    let path_re = Regex::new(r"^  (/[^\s:]+):").unwrap();
    let bound_re = Regex::new(r"x-handler-bound\s*:").unwrap();

    let walker = WalkDir::new(&contracts_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |x| x == "yaml" || x == "yml")
        });

    for entry in walker {
        let path = entry.path();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if let Some(cap) = path_re.captures(lines[i]) {
                let api_path = cap.get(1).unwrap().as_str().to_owned();
                // Look ahead up to 20 lines for x-handler-bound
                let window_end = (i + 20).min(lines.len());
                let window = &lines[i..window_end];
                let has_bound = window.iter().any(|l| bound_re.is_match(l));
                if !has_bound {
                    let mut f = Finding::new(
                        &rel,
                        i + 1,
                        3,
                        "openapi_unbound_path",
                        format!("OpenAPI path `{api_path}` has no x-handler-bound annotation"),
                        Severity::Warning,
                    );
                    if allowlist.is_active(&f.id) {
                        if let Some(e) = allowlist.get(&f.id) {
                            f.allowlisted = true;
                            f.adr = e.adr.clone();
                            f.expires = e.expires.clone();
                        }
                    }
                    findings.push(f);
                }
            }
            i += 1;
        }
    }
    Ok(findings)
}

/// Emit findings and return an exit code (0 = pass, 1 = fail).
pub fn emit(findings: &[Finding], format: &str, fail_on_new: bool, strict: bool) -> Result<i32> {
    let actionable: Vec<&Finding> = if strict {
        findings.iter().collect()
    } else {
        findings.iter().filter(|f| !f.allowlisted).collect()
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(findings)?);
        }
        "junit" => {
            emit_junit(findings)?;
        }
        _ => {
            emit_human(findings, strict);
        }
    }

    let has_errors = actionable
        .iter()
        .any(|f| matches!(f.severity, crate::finding::Severity::Error));
    let should_fail = if fail_on_new || strict {
        !actionable.is_empty()
    } else {
        has_errors
    };

    Ok(if should_fail { 1 } else { 0 })
}

fn emit_human(findings: &[Finding], strict: bool) {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    for f in findings {
        let suppressed = f.allowlisted && !strict;
        let tag = if suppressed { "[ALLOW]" } else { "" };
        eprintln!(
            "{} {}:{} [{} {}] {}",
            f.severity, f.file, f.line, f.kind, tag, f.message
        );
        if !suppressed {
            match f.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
            }
        }
    }
    eprintln!("--- {errors} errors, {warnings} warnings, {infos} infos ---");
}

fn emit_junit(findings: &[Finding]) -> Result<()> {
    use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};

    let mut suite = TestSuite::new("eel-completeness");
    for f in findings {
        let status = if f.allowlisted {
            TestCaseStatus::success()
        } else {
            let mut s = TestCaseStatus::non_success(NonSuccessKind::Failure);
            s.set_message(f.message.clone());
            s.set_description(format!("{}:{}: {}", f.file, f.line, f.message));
            s
        };
        let tc = TestCase::new(format!("{}:{}:{}", f.file, f.line, f.kind), status);
        suite.add_test_case(tc);
    }
    let mut report = Report::new("eel-completeness");
    report.add_test_suite(suite);
    let xml = report.to_string()?;
    print!("{xml}");
    Ok(())
}
