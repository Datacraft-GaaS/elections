use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    domain::{AdhocPreview, AdhocReportVersion, PreviewFormat, PreviewRequest},
    error::{AdhocError, Result},
    validator::validate_sql,
};

const MAX_MEMORY_BYTES: usize = 1_024 * 1_024; // 1 MB
const MAX_SAMPLE_LIMIT: i32 = 1_000;

/// Run a preview. In production this executes against a read-replica PG with
/// statement_timeout = 10s. In tests a fixture vec of rows is used.
pub async fn run_preview(
    version: &AdhocReportVersion,
    req: PreviewRequest,
    requested_by: Uuid,
    rows_provider: impl std::future::Future<Output = Result<Vec<Value>>>,
) -> Result<AdhocPreview> {
    let sample_limit = req.sample_limit.unwrap_or(100);
    if sample_limit > MAX_SAMPLE_LIMIT {
        return Err(AdhocError::Preview(format!(
            "sample_limit {sample_limit} exceeds maximum {MAX_SAMPLE_LIMIT}"
        )));
    }

    let format = req.format.clone().unwrap_or(PreviewFormat::Json);

    // Validate the SQL before running
    let lint = validate_sql(&version.source, &version.allowed_tables, Some(sample_limit as u64))
        .map_err(|e| AdhocError::Preview(e.to_string()))?;
    if !lint.is_safe() {
        return Err(AdhocError::Preview("SQL failed validation".to_string()));
    }

    let start = std::time::Instant::now();
    let rows = rows_provider.await?;
    let elapsed_ms = start.elapsed().as_millis() as i32;

    let rows_returned = rows.len() as i32;

    let preview_bytes = render_preview_rows(&rows, &format, version)?;

    if preview_bytes.len() > MAX_MEMORY_BYTES {
        return Err(AdhocError::MemoryCap { bytes: preview_bytes.len() });
    }

    let now = Utc::now();
    Ok(AdhocPreview {
        preview_id: Uuid::now_v7(),
        version_id: version.version_id,
        parameter_values: req.parameter_values.unwrap_or_else(|| serde_json::json!({})),
        sample_limit,
        rows_returned: Some(rows_returned),
        elapsed_ms: Some(elapsed_ms),
        preview_format: format,
        preview_bytes: Some(preview_bytes),
        requested_by,
        requested_at: now,
        expires_at: now + chrono::Duration::hours(2),
        error_message: None,
    })
}

fn render_preview_rows(
    rows: &[Value],
    format: &PreviewFormat,
    _version: &AdhocReportVersion,
) -> Result<Vec<u8>> {
    match format {
        PreviewFormat::Json => {
            let json = serde_json::to_vec(rows).map_err(|e| AdhocError::Render(e.to_string()))?;
            Ok(json)
        }
        PreviewFormat::Csv => {
            let mut csv = String::new();
            if let Some(first) = rows.first() {
                if let Some(obj) = first.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                    csv.push_str(&headers.join(","));
                    csv.push('\n');
                    for row in rows {
                        if let Some(obj) = row.as_object() {
                            let vals: Vec<String> = headers
                                .iter()
                                .map(|h| {
                                    obj.get(*h)
                                        .map(|v| match v {
                                            Value::String(s) => format!("\"{s}\""),
                                            Value::Null => String::new(),
                                            other => other.to_string(),
                                        })
                                        .unwrap_or_default()
                                })
                                .collect();
                            csv.push_str(&vals.join(","));
                            csv.push('\n');
                        }
                    }
                }
            }
            Ok(csv.into_bytes())
        }
        PreviewFormat::Markdown => {
            use eel_abis_render_markdown::{
                render_markdown, MarkdownOptions, MarkdownTemplate, MdSection, TableAlignment,
            };
            use std::collections::BTreeMap;
            let template = MarkdownTemplate {
                title: "Preview".to_string(),
                front_matter: BTreeMap::new(),
                sections: vec![],
                gfm: true,
                include_toc: false,
                table_alignment: TableAlignment::Left,
            };
            let opts = MarkdownOptions::default();
            let out = render_markdown(rows, &template, &opts)
                .map_err(|e| AdhocError::Render(e.to_string()))?;
            Ok(out.bytes)
        }
        PreviewFormat::Docx => {
            use eel_abis_render_docx::{
                render_docx, DocxOptions, DocxTemplate, PageOrientation, PageSize,
            };
            use std::collections::BTreeMap;
            let template = DocxTemplate {
                title: "Preview".to_string(),
                subtitle: None,
                header_html: None,
                footer_html: None,
                sections: vec![],
                page_orientation: PageOrientation::Portrait,
                page_size: PageSize::A4,
                margins: None,
                font_family: None,
                base_font_size: None,
                table_style_id: None,
            };
            let opts = DocxOptions::default();
            let out = render_docx(rows, &template, &opts)
                .map_err(|e| AdhocError::Render(e.to_string()))?;
            Ok(out.bytes)
        }
    }
}
