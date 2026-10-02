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

use chrono::Utc;

use crate::error::Result;
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render(rows: &Rows, meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        // Build rows as array of objects keyed by column name
        let row_objects: Vec<serde_json::Value> = rows
            .data
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, col) in rows.columns.iter().enumerate() {
                    obj.insert(
                        col.clone(),
                        row.get(i).cloned().unwrap_or(serde_json::Value::Null),
                    );
                }
                serde_json::Value::Object(obj)
            })
            .collect();

        // Compute sha256 of the raw data for the manifest (chicken-and-egg: use placeholder)
        let manifest = serde_json::json!({
            "slug":             meta.slug,
            "template_version": meta.template_version,
            "window_from":      meta.window_from.to_rfc3339(),
            "window_to":        meta.window_to.to_rfc3339(),
            "row_count":        rows.row_count(),
            "generated_at":     Utc::now().to_rfc3339(),
        });

        let payload = serde_json::json!({
            "manifest": manifest,
            "rows": row_objects,
        });

        let bytes = serde_json::to_vec_pretty(&payload)?;
        Ok(RenderOutput::new(bytes))
    }
}
