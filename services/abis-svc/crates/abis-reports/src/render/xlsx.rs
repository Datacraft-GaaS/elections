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

use rust_xlsxwriter::{Format, Workbook, XlsxError};

use crate::error::{ReportError, Result};
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

pub struct XlsxRenderer;

impl Renderer for XlsxRenderer {
    fn render(rows: &Rows, meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&meta.slug[..meta.slug.len().min(31)])
            .map_err(xlsx_err)?;

        // Header format
        let header_fmt = Format::new().set_bold();

        // Write column headers
        for (col_idx, col_name) in rows.columns.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col_idx as u16, col_name.as_str(), &header_fmt)
                .map_err(xlsx_err)?;
        }

        // Write data rows
        for (row_idx, row) in rows.data.iter().enumerate() {
            let excel_row = (row_idx + 1) as u32;
            for (col_idx, val) in row.iter().enumerate() {
                let col = col_idx as u16;
                match val {
                    serde_json::Value::Null => {}
                    serde_json::Value::Bool(b) => {
                        worksheet
                            .write_boolean(excel_row, col, *b)
                            .map_err(xlsx_err)?;
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            worksheet
                                .write_number(excel_row, col, f)
                                .map_err(xlsx_err)?;
                        }
                    }
                    serde_json::Value::String(s) => {
                        worksheet
                            .write_string(excel_row, col, s.as_str())
                            .map_err(xlsx_err)?;
                    }
                    other => {
                        worksheet
                            .write_string(excel_row, col, other.to_string().as_str())
                            .map_err(xlsx_err)?;
                    }
                }
            }
        }

        // Auto-width columns (approximate)
        for col_idx in 0..rows.columns.len() {
            let max_len = rows
                .data
                .iter()
                .filter_map(|r| r.get(col_idx))
                .map(|v| match v {
                    serde_json::Value::String(s) => s.len(),
                    other => other.to_string().len(),
                })
                .max()
                .unwrap_or(0)
                .max(rows.columns.get(col_idx).map(|c| c.len()).unwrap_or(0));
            worksheet
                .set_column_width(col_idx as u16, (max_len as f64 + 2.0).min(50.0))
                .map_err(xlsx_err)?;
        }

        let bytes = workbook.save_to_buffer().map_err(xlsx_err)?;
        Ok(RenderOutput::new(bytes))
    }
}

fn xlsx_err(e: XlsxError) -> ReportError {
    ReportError::Renderer(e.to_string())
}
