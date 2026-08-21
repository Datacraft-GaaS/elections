use std::collections::BTreeMap;

use docx_rs::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::Result;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub text: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

impl Default for PageOrientation {
    fn default() -> Self {
        Self::Portrait
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageSize {
    A4,
    Letter,
}

impl Default for PageSize {
    fn default() -> Self {
        Self::A4
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalloutStyle {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Section {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        runs: Vec<Run>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        column_widths: Option<Vec<u32>>,
    },
    Image {
        bytes: Vec<u8>,
        alt_text: String,
        width_pt: f32,
        height_pt: f32,
    },
    PageBreak,
    TableOfContents {
        max_depth: u8,
    },
    Callout {
        style: CalloutStyle,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxTemplate {
    pub title: String,
    pub subtitle: Option<String>,
    pub header_html: Option<String>,
    pub footer_html: Option<String>,
    pub sections: Vec<Section>,
    #[serde(default)]
    pub page_orientation: PageOrientation,
    #[serde(default)]
    pub page_size: PageSize,
    pub margins: Option<BTreeMap<String, u32>>,
    pub font_family: Option<String>,
    pub base_font_size: Option<u32>,
    pub table_style_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DocxOptions {
    /// Do not auto-generate a data table from rows when true.
    pub suppress_auto_table: bool,
    /// Tera template variables for header/footer rendering.
    pub template_vars: Option<BTreeMap<String, Value>>,
}

#[derive(Debug)]
pub struct DocxOutput {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size_bytes: usize,
    /// Best-effort page count estimate.
    pub page_count: u32,
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render a DOCX document from `rows` + `template`.
pub fn render_docx(
    rows: &[Value],
    template: &DocxTemplate,
    opts: &DocxOptions,
) -> Result<DocxOutput> {
    let mut docx = Docx::new();

    // Title paragraph
    let title_para = Paragraph::new()
        .add_run(
            docx_rs::Run::new()
                .add_text(&template.title)
                .bold()
                .size(48),
        )
        .style("Heading1");
    let title_para = title_para
        .add_bookmark_start(0, "heading_title")
        .add_bookmark_end(0);
    docx = docx.add_paragraph(title_para);

    // Subtitle
    if let Some(sub) = &template.subtitle {
        docx = docx.add_paragraph(
            Paragraph::new().add_run(docx_rs::Run::new().add_text(sub).size(32).color("666666")),
        );
    }

    let mut heading_bookmark_id: usize = 1;

    let has_explicit_table = template
        .sections
        .iter()
        .any(|s| matches!(s, Section::Table { .. }));

    for section in &template.sections {
        match section {
            Section::Heading { level, text } => {
                let style = match level {
                    1 => "Heading1",
                    2 => "Heading2",
                    _ => "Heading3",
                };
                let slug = slugify(text);
                let bm_id = heading_bookmark_id;
                heading_bookmark_id += 1;
                let p = Paragraph::new()
                    .add_bookmark_start(bm_id, &slug)
                    .add_run(docx_rs::Run::new().add_text(text))
                    .add_bookmark_end(bm_id)
                    .style(style);
                docx = docx.add_paragraph(p);
            }
            Section::Paragraph { runs } => {
                let mut p = Paragraph::new();
                for r in runs {
                    let mut dr = docx_rs::Run::new().add_text(&r.text);
                    if r.bold {
                        dr = dr.bold();
                    }
                    if r.italic {
                        dr = dr.italic();
                    }
                    if r.underline {
                        dr = dr.underline("single");
                    }
                    if let Some(c) = &r.color {
                        dr = dr.color(c);
                    }
                    p = p.add_run(dr);
                }
                docx = docx.add_paragraph(p);
            }
            Section::Table {
                headers,
                rows: table_rows,
                column_widths,
            } => {
                docx = add_table_section(docx, headers, table_rows, column_widths.as_deref())?;
            }
            Section::Image {
                bytes, alt_text, ..
            } => {
                let pic = Pic::new(bytes).size(2_000_000, 1_500_000);
                docx = docx
                    .add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_image(pic)));
                // alt text paragraph for accessibility
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(
                        docx_rs::Run::new()
                            .add_text(alt_text)
                            .color("888888")
                            .size(16),
                    ),
                );
            }
            Section::PageBreak => {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(docx_rs::Run::new().add_break(BreakType::Page)),
                );
            }
            Section::TableOfContents { .. } => {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(docx_rs::Run::new().add_text("[Table of Contents]")),
                );
            }
            Section::Callout { style, text } => {
                let color = match style {
                    CalloutStyle::Info => "004085",
                    CalloutStyle::Warn => "856404",
                    CalloutStyle::Error => "721c24",
                };
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(docx_rs::Run::new().add_text(text).color(color).bold())
                        .indent(Some(720), None, None, None),
                );
            }
        }
    }

    // Auto-generate a data-driven table when rows non-empty and no explicit table section
    if !rows.is_empty() && !has_explicit_table && !opts.suppress_auto_table {
        if let Some(first) = rows.first() {
            if let Some(obj) = first.as_object() {
                let headers: Vec<String> = obj.keys().cloned().collect();
                let table_rows: Vec<Vec<String>> = rows
                    .iter()
                    .filter_map(|r| r.as_object())
                    .map(|obj| {
                        headers
                            .iter()
                            .map(|h| obj.get(h).map(|v| value_to_str(v)).unwrap_or_default())
                            .collect()
                    })
                    .collect();
                docx = add_table_section(docx, &headers, &table_rows, None)?;
            }
        }
    }

    // Build the DOCX bytes
    let mut cursor = std::io::Cursor::new(Vec::new());
    docx.build()
        .pack(&mut cursor)
        .map_err(|e| crate::error::RenderDocxError::DocxBuild(e.to_string()))?;
    let bytes = cursor.into_inner();

    let sha256 = hex::encode(Sha256::digest(&bytes));
    let size_bytes = bytes.len();
    let page_count = estimate_page_count(rows.len(), &template.sections);

    Ok(DocxOutput {
        bytes,
        sha256,
        size_bytes,
        page_count,
    })
}

fn add_table_section(
    mut docx: Docx,
    headers: &[String],
    rows: &[Vec<String>],
    _column_widths: Option<&[u32]>,
) -> Result<Docx> {
    let col_count = headers.len();
    let cell_width: usize = if col_count > 0 {
        9000 / col_count
    } else {
        1000
    };

    let header_cells: Vec<TableCell> = headers
        .iter()
        .map(|h| {
            TableCell::new()
                .add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_text(h).bold()))
                .width(cell_width, WidthType::Dxa)
        })
        .collect();

    let mut tbl = Table::new(vec![TableRow::new(header_cells)]);

    for row in rows {
        let cells: Vec<TableCell> = (0..col_count)
            .map(|i| {
                let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_text(val)))
                    .width(cell_width, WidthType::Dxa)
            })
            .collect();
        tbl = tbl.add_row(TableRow::new(cells));
    }

    docx = docx.add_table(tbl);
    Ok(docx)
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn value_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn estimate_page_count(row_count: usize, sections: &[Section]) -> u32 {
    let rows_per_page = 40;
    let section_pages = sections.len() / 10 + 1;
    let table_pages = row_count / rows_per_page + 1;
    (section_pages + table_pages) as u32
}
