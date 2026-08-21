//! Report generation: JSON and HTML output.

use crate::audit::AuditResult;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_urls: usize,
    pub passing: usize,
    pub failing: usize,
    pub errors: usize,
    pub violations_total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub generated_at: String,
    pub summary: ReportSummary,
    pub results: Vec<AuditResult>,
}

impl Report {
    pub fn from_results(results: Vec<AuditResult>) -> Self {
        let passing = results.iter().filter(|r| r.is_passing()).count();
        let failing = results.iter().filter(|r| r.status == "fail").count();
        let errors = results.iter().filter(|r| r.status == "error").count();
        let violations_total = results.iter().map(|r| r.violations.len()).sum();

        Report {
            generated_at: Utc::now().to_rfc3339(),
            summary: ReportSummary {
                total_urls: results.len(),
                passing,
                failing,
                errors,
                violations_total,
            },
            results,
        }
    }

    pub fn has_violations(&self) -> bool {
        self.summary.violations_total > 0 || self.summary.errors > 0
    }
}

pub fn write_json(report: &Report, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn write_html(report: &Report, path: &Path) -> Result<()> {
    let html = render_html(report);
    std::fs::write(path, html)?;
    Ok(())
}

pub fn render_json(report: &Report) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

fn status_badge(status: &str) -> &'static str {
    match status {
        "pass" => "<span class=\"badge pass\">PASS</span>",
        "fail" => "<span class=\"badge fail\">FAIL</span>",
        _ => "<span class=\"badge error\">ERROR</span>",
    }
}

pub fn render_html(report: &Report) -> String {
    let s = &report.summary;
    let rows: String = report
        .results
        .iter()
        .map(|r| {
            let badge = status_badge(&r.status);
            let violation_count = r.violations.len();
            let violations_html = if r.violations.is_empty() {
                String::new()
            } else {
                let items: String = r
                    .violations
                    .iter()
                    .map(|v| {
                        format!(
                            "<li><code>{}</code> {} (impact: {})</li>",
                            v.id,
                            v.description.as_deref().unwrap_or(""),
                            v.impact.as_deref().unwrap_or("unknown")
                        )
                    })
                    .collect();
                format!("<ul class=\"violations\">{items}</ul>")
            };
            let error_html = r
                .error
                .as_deref()
                .map(|e| format!("<p class=\"error-msg\">{e}</p>"))
                .unwrap_or_default();
            format!(
                "<tr>\
                    <td>{badge}</td>\
                    <td><a href=\"{url}\">{name}</a></td>\
                    <td>{module}</td>\
                    <td>{level}</td>\
                    <td>{violation_count}</td>\
                    <td>{passes}</td>\
                </tr>\
                {violations_html}{error_html}",
                url = r.url,
                name = r.name,
                module = r.module,
                level = r.level,
                passes = r.passes,
            )
        })
        .collect();

    let pass_color = if s.failing == 0 && s.errors == 0 {
        "#22c55e"
    } else {
        "#ef4444"
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>eel WCAG Audit Report</title>
<meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root {{
  --bg: #fff; --fg: #111; --muted: #555; --border: #ddd;
  --pass-bg: #dcfce7; --fail-bg: #fee2e2; --error-bg: #fef3c7;
  --pass-fg: #166534; --fail-fg: #991b1b; --error-fg: #92400e;
  --code-bg: #f3f4f6;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    --bg: #0f172a; --fg: #f1f5f9; --muted: #94a3b8; --border: #334155;
    --pass-bg: #14532d; --fail-bg: #7f1d1d; --error-bg: #78350f;
    --pass-fg: #86efac; --fail-fg: #fca5a5; --error-fg: #fcd34d;
    --code-bg: #1e293b;
  }}
}}
body {{ background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; margin: 2rem; }}
h1 {{ font-size: 1.5rem; margin-bottom: 0.25rem; }}
.meta {{ color: var(--muted); font-size: 0.875rem; margin-bottom: 1.5rem; }}
.summary {{ display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.5rem; }}
.stat {{ padding: 0.75rem 1rem; border: 1px solid var(--border); border-radius: 6px; min-width: 100px; }}
.stat-label {{ font-size: 0.75rem; color: var(--muted); text-transform: uppercase; }}
.stat-value {{ font-size: 1.75rem; font-weight: 700; color: {pass_color}; }}
table {{ border-collapse: collapse; width: 100%; margin-bottom: 1rem; }}
th, td {{ border: 1px solid var(--border); padding: 0.5rem 0.75rem; text-align: left; }}
th {{ background: var(--code-bg); font-size: 0.8rem; text-transform: uppercase; }}
.badge {{ padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.75rem; font-weight: 600; }}
.badge.pass {{ background: var(--pass-bg); color: var(--pass-fg); }}
.badge.fail {{ background: var(--fail-bg); color: var(--fail-fg); }}
.badge.error {{ background: var(--error-bg); color: var(--error-fg); }}
ul.violations {{ margin: 0.25rem 0 0.5rem 1rem; font-size: 0.85rem; }}
code {{ background: var(--code-bg); padding: 0.1rem 0.3rem; border-radius: 3px; }}
.error-msg {{ color: var(--fail-fg); font-size: 0.85rem; margin: 0.25rem 0; }}
a {{ color: inherit; }}
</style>
</head>
<body>
<h1>eel WCAG 2.2 Audit Report</h1>
<div class="meta">Generated: {generated_at}</div>
<div class="summary">
  <div class="stat"><div class="stat-label">Total URLs</div><div class="stat-value">{total}</div></div>
  <div class="stat"><div class="stat-label">Passing</div><div class="stat-value" style="color:#22c55e">{passing}</div></div>
  <div class="stat"><div class="stat-label">Failing</div><div class="stat-value" style="color:#ef4444">{failing}</div></div>
  <div class="stat"><div class="stat-label">Violations</div><div class="stat-value" style="color:#f97316">{vcount}</div></div>
</div>
<table>
<thead><tr><th>Status</th><th>Page</th><th>Module</th><th>Level</th><th>Violations</th><th>Passes</th></tr></thead>
<tbody>{rows}</tbody>
</table>
</body>
</html>"#,
        generated_at = report.generated_at,
        total = s.total_urls,
        passing = s.passing,
        failing = s.failing,
        vcount = s.violations_total,
        rows = rows,
    )
}
