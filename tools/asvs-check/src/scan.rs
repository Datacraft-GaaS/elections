use crate::manifest::{Level, Requirement};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TagHit {
    pub tag: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct RequirementResult {
    pub requirement: Requirement,
    pub covered: bool,
    pub hits: Vec<TagHit>,
}

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

pub fn scan_repo(root: &Path) -> Result<HashMap<String, Vec<TagHit>>> {
    let patterns = build_patterns();
    let mut tag_hits: HashMap<String, Vec<TagHit>> = HashMap::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();

        let applicable: Vec<&TagPattern> = patterns
            .iter()
            .filter(|p| p.extensions.contains(&ext))
            .collect();

        if applicable.is_empty() {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (line_no, line) in content.lines().enumerate() {
            for pat in &applicable {
                for caps in pat.re.captures_iter(line) {
                    let raw = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                    for tag in extract_tags(raw) {
                        let location = format!("{}:{}", rel_path, line_no + 1);
                        tag_hits
                            .entry(tag.clone())
                            .or_default()
                            .push(TagHit { tag, location });
                    }
                }
            }
        }
    }

    Ok(tag_hits)
}

pub fn map_requirements(
    requirements: &[Requirement],
    tag_hits: &HashMap<String, Vec<TagHit>>,
) -> Vec<RequirementResult> {
    requirements
        .iter()
        .map(|req| {
            let mut hits = Vec::new();

            // Direct ASVS tag match: ASVS-V6.2.3
            let direct_tag = format!("ASVS-{}", req.id);
            if let Some(h) = tag_hits.get(&direct_tag) {
                hits.extend_from_slice(h);
            }

            // Section prefix match: ASVS-V6 covers all V6.x.y
            let section = req.id.split('.').next().unwrap_or(&req.id);
            let section_tag = format!("ASVS-{}", section);
            if let Some(h) = tag_hits.get(&section_tag) {
                hits.extend_from_slice(h);
            }

            // Mapped tags from manifest
            for mapped in &req.mapped_tags {
                if let Some(h) = tag_hits.get(mapped) {
                    hits.extend_from_slice(h);
                }
            }

            hits.dedup_by(|a, b| a.location == b.location);

            let covered = !hits.is_empty();
            RequirementResult {
                requirement: req.clone(),
                covered,
                hits,
            }
        })
        .collect()
}

pub fn gaps(results: &[RequirementResult]) -> Vec<&RequirementResult> {
    results.iter().filter(|r| !r.covered).collect()
}

pub fn gaps_by_level<'a>(
    results: &'a [RequirementResult],
    level: &Level,
) -> Vec<&'a RequirementResult> {
    results
        .iter()
        .filter(|r| !r.covered && &r.requirement.level == level)
        .collect()
}

fn extract_tags(raw: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for part in raw.split(',') {
        let t = part.trim().trim_matches('"').trim_matches('\'').trim();
        if !t.is_empty() {
            tags.push(t.to_string());
        }
    }
    tags
}

struct TagPattern {
    re: Regex,
    extensions: Vec<&'static str>,
}

fn build_patterns() -> Vec<TagPattern> {
    vec![
        // Rust: #[trace("TAG1", "TAG2")]
        TagPattern {
            re: Regex::new(r#"#\[trace\(([^)]+)\)\]"#).unwrap(),
            extensions: vec!["rs"],
        },
        // Python: @pytest.mark.trace("TAG1", "TAG2")
        TagPattern {
            re: Regex::new(r#"@pytest\.mark\.trace\(([^)]+)\)"#).unwrap(),
            extensions: vec!["py"],
        },
        // ExUnit: @tag trace: ["TAG1", "TAG2"]
        TagPattern {
            re: Regex::new(r#"@tag trace: \[([^\]]+)\]"#).unwrap(),
            extensions: vec!["ex", "exs"],
        },
        // Flutter/Dart: testWidgets('desc [TAG]')
        TagPattern {
            re: Regex::new(r#"\[([A-Z]{2,}-[A-Z0-9][A-Z0-9.-]*)\]"#).unwrap(),
            extensions: vec!["dart"],
        },
        // Playwright/JS/TS: test('desc @TAG')
        TagPattern {
            re: Regex::new(r#"@([A-Z]{2,}-[A-Z0-9][A-Z0-9.-]*)"#).unwrap(),
            extensions: vec!["ts", "js"],
        },
    ]
}
