use printpdf::{Mm, PdfDocument, PdfLayerReference};

use crate::error::{ReportError, Result};
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

const ROWS_PER_PAGE: usize = 40;
// A4 landscape in mm (f32 required by printpdf)
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
    fn render(rows: &Rows, meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        let title = format!("{} \u{2014} {}", meta.slug, meta.title);
        let (doc, page1, layer1) =
            PdfDocument::new(title.clone(), Mm(PAGE_W), Mm(PAGE_H), "Layer 1");

        let font = doc
            .add_builtin_font(printpdf::BuiltinFont::Helvetica)
            .map_err(|e| ReportError::Renderer(e.to_string()))?;
        let font_bold = doc
            .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
            .map_err(|e| ReportError::Renderer(e.to_string()))?;

        let first_layer = doc.get_page(page1).get_layer(layer1);
        render_page_header(&first_layer, meta, &font_bold, &title)?;
        render_table_header(&first_layer, &rows.columns, &font_bold)?;

        let mut page_num: usize = 1;
        let mut layer_ref = first_layer;

        for (chunk_idx, chunk) in rows.data.chunks(ROWS_PER_PAGE).enumerate() {
            if chunk_idx > 0 {
                page_num += 1;
                let (page_idx, layer_idx) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Layer 1");
                layer_ref = doc.get_page(page_idx).get_layer(layer_idx);
                render_page_header(&layer_ref, meta, &font_bold, &title)?;
                render_table_header(&layer_ref, &rows.columns, &font_bold)?;
            }
            render_table_body(&layer_ref, chunk, &rows.columns, &font)?;
            render_footer(&layer_ref, page_num, &meta.slug, &font)?;
        }

        if rows.data.is_empty() {
            render_footer(&layer_ref, 1, &meta.slug, &font)?;
        }

        let bytes = doc
            .save_to_bytes()
            .map_err(|e| ReportError::Renderer(e.to_string()))?;
        Ok(RenderOutput::new(bytes))
    }
}

fn render_page_header(
    layer: &PdfLayerReference,
    meta: &RunMeta,
    font: &printpdf::IndirectFontRef,
    title: &str,
) -> Result<()> {
    layer.use_text(title, 14.0, Mm(MARGIN), Mm(HEADER_Y), font);
    let window = format!(
        "{} to {}",
        meta.window_from.format("%Y-%m-%d"),
        meta.window_to.format("%Y-%m-%d")
    );
    layer.use_text(window.as_str(), 9.0, Mm(MARGIN), Mm(SUBTITLE_Y), font);
    Ok(())
}

fn render_table_header(
    layer: &PdfLayerReference,
    columns: &[String],
    font: &printpdf::IndirectFontRef,
) -> Result<()> {
    if columns.is_empty() {
        return Ok(());
    }
    let col_w = (PAGE_W - MARGIN * 2.0) / columns.len() as f32;
    for (i, col) in columns.iter().enumerate() {
        let x = MARGIN + i as f32 * col_w;
        layer.use_text(col.as_str(), 8.0, Mm(x), Mm(TABLE_HDR_Y), font);
    }
    Ok(())
}

fn render_table_body(
    layer: &PdfLayerReference,
    chunk: &[Vec<serde_json::Value>],
    columns: &[String],
    font: &printpdf::IndirectFontRef,
) -> Result<()> {
    if columns.is_empty() {
        return Ok(());
    }
    let col_w = (PAGE_W - MARGIN * 2.0) / columns.len() as f32;
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
            layer.use_text(truncated.as_str(), 7.0, Mm(x), Mm(y), font);
        }
    }
    Ok(())
}

fn render_footer(
    layer: &PdfLayerReference,
    page: usize,
    slug: &str,
    font: &printpdf::IndirectFontRef,
) -> Result<()> {
    let footer = format!("Page {page} | {slug} | tlog: pending");
    layer.use_text(footer.as_str(), 7.0, Mm(MARGIN), Mm(FOOTER_Y), font);
    Ok(())
}
