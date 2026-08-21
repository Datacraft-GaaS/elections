use std::collections::HashMap;

use tera::{Context, Tera};

use crate::error::Result;
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

pub struct HtmlRenderer;

impl Renderer for HtmlRenderer {
    fn render(rows: &Rows, meta: &RunMeta, template: &TemplateBundle) -> Result<RenderOutput> {
        let mut ctx = Context::new();
        ctx.insert("slug", &meta.slug);
        ctx.insert("title", &meta.title);
        ctx.insert("template_version", &meta.template_version);
        ctx.insert("window_from", &meta.window_from.to_rfc3339());
        ctx.insert("window_to", &meta.window_to.to_rfc3339());
        ctx.insert("generated_at", &meta.generated_at.to_rfc3339());
        ctx.insert("row_count", &rows.row_count());
        ctx.insert("columns", &rows.columns);
        ctx.insert("chart_svg", &template.chart_svg);

        let row_maps: Vec<HashMap<String, serde_json::Value>> = rows
            .data
            .iter()
            .map(|row| {
                rows.columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| {
                        (
                            col.clone(),
                            row.get(i).cloned().unwrap_or(serde_json::Value::Null),
                        )
                    })
                    .collect()
            })
            .collect();
        ctx.insert("rows", &row_maps);

        let html = if let Some(path) = &template.html_template_path {
            let mut tera = Tera::default();
            tera.add_template_file(path, Some("report"))?;
            tera.render("report", &ctx)?
        } else {
            render_inline(rows, meta)?
        };

        Ok(RenderOutput::new(html.into_bytes()))
    }
}

/// Inline HTML fallback — no external assets, CSP meta, WCAG AA landmarks.
fn render_inline(rows: &Rows, meta: &RunMeta) -> Result<String> {
    let title = format!("{} \u{2014} {}", meta.slug, meta.title);
    let col_headers: String = rows
        .columns
        .iter()
        .map(|c| format!("<th>{}</th>", he(c)))
        .collect::<Vec<_>>()
        .join("");

    let data_rows: String = rows
        .data
        .iter()
        .map(|row| {
            let cells: String = rows
                .columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let val = row
                        .get(i)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();
                    format!("<td>{}</td>", he(&val))
                })
                .collect::<Vec<_>>()
                .join("");
            format!("<tr>{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let window = format!(
        "{} to {}",
        meta.window_from.format("%Y-%m-%d"),
        meta.window_to.format("%Y-%m-%d")
    );

    // Build the HTML by concatenation to avoid raw-string delimiter conflicts
    // (href="#..." and "##" in CSS would terminate r##-delimited strings).
    let mut html = String::with_capacity(4096);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\" />\n");
    html.push_str("  <meta http-equiv=\"Content-Security-Policy\"\n");
    html.push_str(
		"        content=\"default-src 'self'; style-src 'unsafe-inline'; img-src 'self' data:; script-src 'none'\" />\n",
	);
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n");
    html.push_str(&format!("  <title>{}</title>\n", he(&title)));
    html.push_str("  <style>\n");
    html.push_str(
        "    body { font-family: sans-serif; margin: 2rem; color: #111; background: #fff; }\n",
    );
    html.push_str("    h1 { font-size: 1.4rem; }\n");
    html.push_str("    .meta { font-size: 0.85rem; color: #555; margin-bottom: 1rem; }\n");
    html.push_str("    table { border-collapse: collapse; width: 100%; font-size: 0.9rem; }\n");
    html.push_str(
        "    th,td { border: 1px solid #ccc; padding: 0.4rem 0.6rem; text-align: left; }\n",
    );
    html.push_str("    th { background: #f0f0f0; font-weight: bold; }\n");
    html.push_str("    tr:nth-child(even) { background: #fafafa; }\n");
    html.push_str("    a:focus, button:focus { outline: 3px solid #005fcc; }\n");
    html.push_str("    .skip-link { position: absolute; left: -999px; top: 0; }\n");
    html.push_str(
        "    .skip-link:focus { left: 0; background: #005fcc; color: #fff; padding: 0.5rem; }\n",
    );
    html.push_str("  </style>\n</head>\n<body>\n");
    html.push_str("  <a href=\"#main-content\" class=\"skip-link\">Skip to main content</a>\n");
    html.push_str("  <header role=\"banner\">\n");
    html.push_str(&format!("    <h1>{}</h1>\n", he(&title)));
    html.push_str(&format!(
        "    <div class=\"meta\">Window: {} | Rows: {} | Generated: {}</div>\n",
        he(&window),
        rows.row_count(),
        he(&meta.generated_at.to_rfc3339()),
    ));
    html.push_str("  </header>\n");
    html.push_str("  <main id=\"main-content\" role=\"main\">\n");
    html.push_str("    <section aria-label=\"Report data\">\n");
    html.push_str("      <table>\n        <thead>\n          <tr>");
    html.push_str(&col_headers);
    html.push_str("</tr>\n        </thead>\n        <tbody>\n");
    html.push_str(&data_rows);
    html.push_str("\n        </tbody>\n      </table>\n    </section>\n  </main>\n");
    html.push_str("  <footer role=\"contentinfo\">\n");
    html.push_str(&format!(
        "    <small>Slug: {} | Version: {}</small>\n",
        he(&meta.slug),
        he(&meta.template_version),
    ));
    html.push_str("  </footer>\n</body>\n</html>\n");

    Ok(html)
}

/// Minimal HTML escaping.
fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
