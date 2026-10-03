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

use crate::error::Result;
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

pub struct CsvRenderer;

impl Renderer for CsvRenderer {
    fn render(rows: &Rows, _meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(vec![]);

        // UTF-8 BOM for Excel compat
        let bom: &[u8] = b"\xEF\xBB\xBF";

        // Header row
        wtr.write_record(&rows.columns)?;

        // Data rows
        for row in &rows.data {
            let record: Vec<String> = row.iter().map(json_to_csv_string).collect();
            wtr.write_record(&record)?;
        }

        let csv_bytes = wtr
            .into_inner()
            .map_err(|e| crate::error::ReportError::Renderer(e.to_string()))?;

        let mut bytes = Vec::with_capacity(bom.len() + csv_bytes.len());
        bytes.extend_from_slice(bom);
        bytes.extend_from_slice(&csv_bytes);

        Ok(RenderOutput::new(bytes))
    }
}

fn json_to_csv_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => v.to_string(),
    }
}
