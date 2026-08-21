use crate::manifest::Level;
use crate::scan::RequirementResult;
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
pub struct JsonReport {
    pub generated_at: String,
    pub summary: Summary,
    pub requirements: Vec<JsonRequirement>,
    pub gaps: Vec<JsonGap>,
}

#[derive(Serialize)]
pub struct Summary {
    pub l3_total: usize,
    pub l3_covered: usize,
    pub l2_total: usize,
    pub l2_covered: usize,
}

#[derive(Serialize)]
pub struct JsonRequirement {
    pub id: String,
    pub level: String,
    pub description: String,
    pub covered: bool,
    pub tests: Vec<String>,
}

#[derive(Serialize)]
pub struct JsonGap {
    pub id: String,
    pub level: String,
    pub description: String,
}

pub fn build_json(results: &[RequirementResult]) -> JsonReport {
    let l3_total = results
        .iter()
        .filter(|r| r.requirement.level == Level::L3)
        .count();
    let l3_covered = results
        .iter()
        .filter(|r| r.requirement.level == Level::L3 && r.covered)
        .count();
    let l2_total = results
        .iter()
        .filter(|r| r.requirement.level == Level::L2)
        .count();
    let l2_covered = results
        .iter()
        .filter(|r| r.requirement.level == Level::L2 && r.covered)
        .count();

    let requirements = results
        .iter()
        .map(|r| JsonRequirement {
            id: r.requirement.id.clone(),
            level: r.requirement.level.to_string(),
            description: r.requirement.description.clone(),
            covered: r.covered,
            tests: r.hits.iter().map(|h| h.location.clone()).collect(),
        })
        .collect();

    let gaps = results
        .iter()
        .filter(|r| !r.covered)
        .map(|r| JsonGap {
            id: r.requirement.id.clone(),
            level: r.requirement.level.to_string(),
            description: r.requirement.description.clone(),
        })
        .collect();

    JsonReport {
        generated_at: Utc::now().to_rfc3339(),
        summary: Summary {
            l3_total,
            l3_covered,
            l2_total,
            l2_covered,
        },
        requirements,
        gaps,
    }
}

pub fn render_json(results: &[RequirementResult]) -> Result<String> {
    let report = build_json(results);
    Ok(serde_json::to_string_pretty(&report)?)
}

pub fn render_markdown(results: &[RequirementResult]) -> String {
    let report = build_json(results);
    let mut out = String::new();

    out.push_str("# ASVS Compliance Report\n\n");
    out.push_str(&format!("Generated: {}\n\n", report.generated_at));
    out.push_str("## Summary\n\n");
    out.push_str("| Level | Total | Covered | Gap |\n");
    out.push_str("|-------|-------|---------|-----|\n");
    out.push_str(&format!(
        "| L3 (election-critical) | {} | {} | {} |\n",
        report.summary.l3_total,
        report.summary.l3_covered,
        report.summary.l3_total - report.summary.l3_covered
    ));
    out.push_str(&format!(
        "| L2 (routine) | {} | {} | {} |\n",
        report.summary.l2_total,
        report.summary.l2_covered,
        report.summary.l2_total - report.summary.l2_covered
    ));
    out.push('\n');

    out.push_str("## Requirements\n\n");
    out.push_str("| ID | Level | Covered | Tests | Description |\n");
    out.push_str("|----|-------|---------|-------|-------------|\n");
    for req in &report.requirements {
        let covered = if req.covered { "yes" } else { "NO" };
        let tests = if req.tests.is_empty() {
            String::from("none")
        } else {
            req.tests.join(", ")
        };
        let desc_short: String = req.description.chars().take(80).collect();
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            req.id, req.level, covered, tests, desc_short
        ));
    }
    out.push('\n');

    if !report.gaps.is_empty() {
        out.push_str("## Gaps\n\n");
        for gap in &report.gaps {
            out.push_str(&format!(
                "- **{}** ({}): {}\n",
                gap.id, gap.level, gap.description
            ));
        }
    }

    out
}

pub fn render_html(results: &[RequirementResult]) -> String {
    let report = build_json(results);
    let l3_pct = if report.summary.l3_total > 0 {
        (report.summary.l3_covered * 100) / report.summary.l3_total
    } else {
        0
    };
    let l2_pct = if report.summary.l2_total > 0 {
        (report.summary.l2_covered * 100) / report.summary.l2_total
    } else {
        0
    };

    let mut rows = String::new();
    for req in &report.requirements {
        let row_class = if req.covered { "covered" } else { "gap" };
        let covered_str = if req.covered { "yes" } else { "NO" };
        let tests_str = if req.tests.is_empty() {
            String::from("none")
        } else {
            req.tests.join("<br>")
        };
        rows.push_str(&format!(
            "<tr class=\"{row_class}\"><td>{id}</td><td>{level}</td><td>{covered}</td><td>{tests}</td><td>{desc}</td></tr>\n",
            row_class = row_class,
            id = req.id,
            level = req.level,
            covered = covered_str,
            tests = tests_str,
            desc = html_escape(&req.description),
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>ASVS Compliance Report</title>
<style>
body {{ font-family: sans-serif; margin: 2em; }}
h1 {{ color: #1a1a2e; }}
.stat {{ display: inline-block; margin: 1em; padding: 1em; border: 1px solid #ccc; border-radius: 4px; text-align: center; min-width: 160px; }}
.stat .num {{ font-size: 2em; font-weight: bold; }}
.stat .pct {{ color: #555; }}
table {{ border-collapse: collapse; width: 100%; margin-top: 2em; }}
th {{ background: #1a1a2e; color: white; padding: 8px; text-align: left; }}
td {{ padding: 6px 8px; border-bottom: 1px solid #eee; vertical-align: top; }}
tr.gap {{ background: #fff3f3; }}
tr.covered {{ background: #f3fff3; }}
</style>
</head>
<body>
<h1>ASVS 4.0 Compliance Report</h1>
<p>Generated: {generated_at}</p>
<div class="stat">
  <div>L3 Election-Critical</div>
  <div class="num">{l3_covered}/{l3_total}</div>
  <div class="pct">{l3_pct}% covered</div>
</div>
<div class="stat">
  <div>L2 Routine</div>
  <div class="num">{l2_covered}/{l2_total}</div>
  <div class="pct">{l2_pct}% covered</div>
</div>
<table>
<tr><th>ID</th><th>Level</th><th>Covered</th><th>Tests</th><th>Description</th></tr>
{rows}
</table>
</body>
</html>"#,
        generated_at = report.generated_at,
        l3_covered = report.summary.l3_covered,
        l3_total = report.summary.l3_total,
        l3_pct = l3_pct,
        l2_covered = report.summary.l2_covered,
        l2_total = report.summary.l2_total,
        l2_pct = l2_pct,
        rows = rows,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
