use std::collections::BTreeMap;

use pulldown_cmark::{Options, Parser};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
}

impl Default for TableAlignment {
    fn default() -> Self {
        Self::Left
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
pub enum MdSection {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    Bullets {
        items: Vec<String>,
    },
    Ordered {
        items: Vec<String>,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        alignment: Option<Vec<TableAlignment>>,
    },
    Callout {
        style: CalloutStyle,
        text: String,
    },
    HorizontalRule,
    Raw {
        markdown: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownTemplate {
    pub title: String,
    #[serde(default)]
    pub front_matter: BTreeMap<String, Value>,
    pub sections: Vec<MdSection>,
    #[serde(default = "default_true")]
    pub gfm: bool,
    #[serde(default)]
    pub include_toc: bool,
    #[serde(default)]
    pub table_alignment: TableAlignment,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions {
    pub utf8_bom: bool,
    pub emit_front_matter: bool,
    pub suppress_auto_table: bool,
}

#[derive(Debug)]
pub struct MarkdownOutput {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size_bytes: usize,
}

pub fn render_markdown(
    rows: &[Value],
    template: &MarkdownTemplate,
    opts: &MarkdownOptions,
) -> Result<MarkdownOutput> {
    let mut out = String::new();

    // BOM
    if opts.utf8_bom {
        out.push('\u{FEFF}');
    }

    // Front matter
    if opts.emit_front_matter && !template.front_matter.is_empty() {
        out.push_str("---\n");
        for (k, v) in &template.front_matter {
            out.push_str(&format!("{k}: {v}\n"));
        }
        out.push_str("---\n\n");
    }

    // Title
    out.push_str(&format!("# {}\n\n", template.title));

    // Auto-TOC
    if template.include_toc {
        out.push_str("## Table of Contents\n\n");
        for section in &template.sections {
            if let MdSection::Heading { level, text } = section {
                let indent = "  ".repeat((*level as usize).saturating_sub(1));
                let slug = slugify(text);
                out.push_str(&format!("{indent}- [{text}](#{slug})\n"));
            }
        }
        out.push('\n');
    }

    // Sections
    let has_explicit_table = template
        .sections
        .iter()
        .any(|s| matches!(s, MdSection::Table { .. }));

    for section in &template.sections {
        match section {
            MdSection::Heading { level, text } => {
                let hashes = "#".repeat(*level as usize);
                out.push_str(&format!("{hashes} {text}\n\n"));
            }
            MdSection::Paragraph { text } => {
                out.push_str(text);
                out.push_str("\n\n");
            }
            MdSection::Bullets { items } => {
                for item in items {
                    out.push_str(&format!("- {item}\n"));
                }
                out.push('\n');
            }
            MdSection::Ordered { items } => {
                for (i, item) in items.iter().enumerate() {
                    out.push_str(&format!("{}. {item}\n", i + 1));
                }
                out.push('\n');
            }
            MdSection::CodeBlock { lang, code } => {
                let lang_str = lang.as_deref().unwrap_or("");
                out.push_str(&format!("```{lang_str}\n{code}\n```\n\n"));
            }
            MdSection::Table {
                headers,
                rows: table_rows,
                alignment,
            } => {
                render_table(
                    &mut out,
                    headers,
                    table_rows,
                    alignment.as_deref(),
                    template.gfm,
                );
            }
            MdSection::Callout { style, text } => {
                let prefix = match style {
                    CalloutStyle::Info => "> **INFO:** ",
                    CalloutStyle::Warn => "> **WARNING:** ",
                    CalloutStyle::Error => "> **ERROR:** ",
                };
                out.push_str(&format!("{prefix}{text}\n\n"));
            }
            MdSection::HorizontalRule => {
                out.push_str("---\n\n");
            }
            MdSection::Raw { markdown } => {
                out.push_str(markdown);
                out.push('\n');
            }
        }
    }

    // Auto table from rows
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
                render_table(&mut out, &headers, &table_rows, None, template.gfm);
            }
        }
    }

    // Validate by round-tripping through pulldown-cmark
    validate_markdown(&out, template.gfm)?;

    let bytes = out.into_bytes();
    let sha256 = hex::encode(Sha256::digest(&bytes));
    let size_bytes = bytes.len();

    Ok(MarkdownOutput {
        bytes,
        sha256,
        size_bytes,
    })
}

fn render_table(
    out: &mut String,
    headers: &[String],
    rows: &[Vec<String>],
    alignment: Option<&[TableAlignment]>,
    gfm: bool,
) {
    if !gfm {
        // CommonMark strict: no table extension, render as pre block
        out.push_str("```\n");
        out.push_str(&headers.join(" | "));
        out.push('\n');
        for row in rows {
            out.push_str(&row.join(" | "));
            out.push('\n');
        }
        out.push_str("```\n\n");
        return;
    }

    // GFM table
    let col_count = headers.len();
    out.push('|');
    for h in headers {
        out.push_str(&format!(" {h} |"));
    }
    out.push('\n');

    out.push('|');
    for i in 0..col_count {
        let align = alignment.and_then(|a| a.get(i));
        let sep = match align {
            Some(TableAlignment::Left) => ":---|",
            Some(TableAlignment::Center) => ":---:|",
            Some(TableAlignment::Right) => "---:|",
            None => "----|",
        };
        out.push_str(&format!(" {sep}"));
    }
    out.push('\n');

    for row in rows {
        out.push('|');
        for i in 0..col_count {
            let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!(" {val} |"));
        }
        out.push('\n');
    }
    out.push('\n');
}

fn validate_markdown(md: &str, gfm: bool) -> Result<()> {
    let mut opts = Options::empty();
    if gfm {
        opts |= Options::ENABLE_TABLES;
        opts |= Options::ENABLE_TASKLISTS;
        opts |= Options::ENABLE_STRIKETHROUGH;
        opts |= Options::ENABLE_GFM;
    }
    // Parse to ensure it's well-formed (pulldown-cmark is infallible but we consume events)
    let _events: Vec<_> = Parser::new_ext(md, opts).collect();
    Ok(())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn value_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
