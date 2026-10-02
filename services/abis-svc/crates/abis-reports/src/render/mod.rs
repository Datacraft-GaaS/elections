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

pub mod csv;
pub mod html;
pub mod json;
pub mod parquet;
pub mod pdf;
pub mod png;
pub mod xlsx;

use crate::error::Result;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Row data abstraction passed to renderers
// ---------------------------------------------------------------------------

/// Opaque row data extracted from SQL. Column names come first, then rows.
#[derive(Debug, Clone)]
pub struct Rows {
	pub columns: Vec<String>,
	pub data:    Vec<Vec<serde_json::Value>>,
}

impl Rows {
	pub fn new(columns: Vec<String>, data: Vec<Vec<serde_json::Value>>) -> Self {
		Self { columns, data }
	}

	pub fn row_count(&self) -> usize {
		self.data.len()
	}
}

/// Metadata about the run, passed to every renderer.
#[derive(Debug, Clone)]
pub struct RunMeta {
	pub slug:             String,
	pub title:            String,
	pub template_version: String,
	pub window_from:      chrono::DateTime<chrono::Utc>,
	pub window_to:        chrono::DateTime<chrono::Utc>,
	pub generated_at:     chrono::DateTime<chrono::Utc>,
}

/// Template bundle loaded from disk.
#[derive(Debug, Clone, Default)]
pub struct TemplateBundle {
	/// Path to the HTML Tera template (if applicable)
	pub html_template_path: Option<String>,
	/// Inline SVG chart string (if applicable)
	pub chart_svg:          Option<String>,
}

/// Result of a render operation.
pub struct RenderOutput {
	pub bytes:      Vec<u8>,
	pub sha256:     Vec<u8>,
	pub size_bytes: i64,
}

impl RenderOutput {
	pub fn new(bytes: Vec<u8>) -> Self {
		let sha256 = Sha256::digest(&bytes).to_vec();
		let size_bytes = bytes.len() as i64;
		Self { bytes, sha256, size_bytes }
	}
}

/// Renderer trait all format renderers must implement.
pub trait Renderer {
	fn render(rows: &Rows, meta: &RunMeta, template: &TemplateBundle) -> Result<RenderOutput>;
}
