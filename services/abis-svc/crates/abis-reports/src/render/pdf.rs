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

use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

const ROWS_PER_PAGE: usize = 40;
// A4 landscape in mm
const PAGE_W: f32 = 297.0;
const PAGE_H: f32 = 210.0;
const MARGIN: f32 = 10.0;
const HEADER_Y: f32 = 196.0;
const SUBTITLE_Y: f32 = 190.0;
const TABLE_HDR_Y: f32 = 183.0;
const TABLE_BODY_Y: f32 = 178.0;
const ROW_H: f32 = 4.0;
const FOOTER_Y: f32 = 5.0;

pub struct PdfRenderer;

impl Renderer for PdfRenderer {
    fn render(rows: &Rows, meta: &RunMeta, _template: &TemplateBundle) -> crate::error::Result<RenderOutput> {
        let title = format!("{} \u{2014} {}", meta.slug, meta.title);

        let mut pages: Vec<PdfPage> = Vec::new();
        let mut ops: Vec<Op> = Vec::new();
        let mut page_num: usize = 1;

        ops.extend(page_header(&meta, &title));
        ops.extend(table_header(&rows.columns));

        for (chunk_idx, chunk) in rows.data.chunks(ROWS_PER_PAGE).enumerate() {
            if chunk_idx > 0 {
                ops.extend(footer(page_num, &meta.slug));
                pages.push(PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops));
                ops = Vec::new();
                page_num += 1;
                ops.extend(page_header(&meta, &title));
                ops.extend(table_header(&rows.columns));
            }
            ops.extend(table_body(chunk, &rows.columns));
        }

        ops.extend(footer(page_num, &meta.slug));
        pages.push(PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops));

        let mut doc = PdfDocument::new(&title);
        let mut warnings = Vec::new();
        let bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut warnings);
        Ok(RenderOutput::new(bytes))
    }
}

/// Emit one line of text at an absolute position.
fn text(x: f32, y: f32, size: f32, bold: bool, s: &str) -> Vec<Op> {
    let font = if bold {
        BuiltinFont::HelveticaBold
    } else {
        BuiltinFont::Helvetica
    };
    vec![
        Op::StartTextSection,
        Op::SetFont {
            font: PdfFontHandle::Builtin(font),
            size: Pt(size),
        },
        Op::SetTextCursor {
            pos: Point::new(Mm(x), Mm(y)),
        },
        Op::ShowText {
            items: vec![TextItem::Text(s.to_string())],
        },
        Op::EndTextSection,
    ]
}

fn page_header(meta: &RunMeta, title: &str) -> Vec<Op> {
    let mut ops = text(MARGIN, HEADER_Y, 14.0, true, title);
    let window = format!(
        "{} to {}",
        meta.window_from.format("%Y-%m-%d"),
        meta.window_to.format("%Y-%m-%d")
    );
    ops.extend(text(MARGIN, SUBTITLE_Y, 9.0, true, &window));
    ops
}

fn table_header(columns: &[String]) -> Vec<Op> {
    if columns.is_empty() {
        return Vec::new();
    }
    let col_w = (PAGE_W - MARGIN * 2.0) / columns.len() as f32;
    let mut ops = Vec::new();
    for (i, col) in columns.iter().enumerate() {
        let x = MARGIN + i as f32 * col_w;
        ops.extend(text(x, TABLE_HDR_Y, 8.0, true, col));
    }
    ops
}

fn table_body(chunk: &[Vec<serde_json::Value>], columns: &[String]) -> Vec<Op> {
    if columns.is_empty() {
        return Vec::new();
    }
    let col_w = (PAGE_W - MARGIN * 2.0) / columns.len() as f32;
    let mut ops = Vec::new();
    for (row_idx, row) in chunk.iter().enumerate() {
        let y = TABLE_BODY_Y - row_idx as f32 * ROW_H;
        for (col_idx, _) in columns.iter().enumerate() {
            let x = MARGIN + col_idx as f32 * col_w;
            let val = row
                .get(col_idx)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            let truncated: String = val.chars().take(20).collect();
            ops.extend(text(x, y, 7.0, false, &truncated));
        }
    }
    ops
}

fn footer(page: usize, slug: &str) -> Vec<Op> {
    let footer = format!("Page {page} | {slug} | tlog: pending");
    text(MARGIN, FOOTER_Y, 7.0, false, &footer)
}
