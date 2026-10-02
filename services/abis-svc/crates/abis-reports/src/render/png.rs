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

use std::io::Read;

use plotters::prelude::*;
use tempfile::NamedTempFile;

use crate::error::{ReportError, Result};
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

const IMG_WIDTH: u32 = 1200;
const IMG_HEIGHT: u32 = 800;

pub struct PngRenderer;

impl Renderer for PngRenderer {
    fn render(rows: &Rows, meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        // Write to a temp PNG file then read it back (.png suffix required by plotters)
        let tmp = tempfile::Builder::new().suffix(".png").tempfile()?;
        let path = tmp.path().to_owned();

        {
            let root = BitMapBackend::new(&path, (IMG_WIDTH, IMG_HEIGHT)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| ReportError::Renderer(e.to_string()))?;

            let numeric_col = rows.columns.iter().position(|_| true);

            if let Some(col_idx) = numeric_col {
                let values: Vec<(usize, f64)> = rows
                    .data
                    .iter()
                    .enumerate()
                    .filter_map(|(i, row)| {
                        row.get(col_idx).and_then(|v| v.as_f64()).map(|f| (i, f))
                    })
                    .collect();

                if !values.is_empty() {
                    let max_val = values
                        .iter()
                        .map(|(_, v)| *v)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let min_val = values.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
                    let y_max = if (max_val - min_val).abs() < f64::EPSILON {
                        max_val + 1.0
                    } else {
                        max_val
                    };

                    let title = format!("{} — {}", meta.slug, meta.template_version);
                    let mut chart = ChartBuilder::on(&root)
                        .caption(&title, ("sans-serif", 20))
                        .margin(40)
                        .x_label_area_size(30)
                        .y_label_area_size(60)
                        .build_cartesian_2d(0usize..values.len(), min_val..y_max)
                        .map_err(|e| ReportError::Renderer(e.to_string()))?;

                    chart
                        .configure_mesh()
                        .draw()
                        .map_err(|e| ReportError::Renderer(e.to_string()))?;

                    chart
                        .draw_series(LineSeries::new(values, &BLUE))
                        .map_err(|e| ReportError::Renderer(e.to_string()))?;
                } else {
                    root.draw_text(
                        "No numeric data",
                        &TextStyle::from(("sans-serif", 20).into_font()).color(&BLACK),
                        (IMG_WIDTH as i32 / 4, IMG_HEIGHT as i32 / 2),
                    )
                    .map_err(|e| ReportError::Renderer(e.to_string()))?;
                }
            }

            root.present()
                .map_err(|e| ReportError::Renderer(e.to_string()))?;
        }

        // Read back PNG bytes
        let mut f = std::fs::File::open(&path)?;
        let mut png_bytes = Vec::new();
        f.read_to_end(&mut png_bytes)?;

        Ok(RenderOutput::new(png_bytes))
    }
}
