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

//! mindtct — NIST NBIS minutiae extraction from raw fingerprint images.
//!
//! Algorithm pipeline (Bazen & Gerez, NIST-Developed):
//! 1. Preprocessing: noise removal, image normalization
//! 2. Orientation field: gradient-based ridge direction estimation
//! 3. Ridge frequency map: local frequency analysis
//! 4. Segmentation: foreground/background mask
//! 5. Binarization: thresholding + thinning (skeletonization)
//! 6. Minutiae detection: ridge endings + bifurcations
//! 7. Quality: reliability map (Bazen & Gerez)
//! 8. Postprocessing: remove spurious minutiae, classify type (end/bifurcation)

use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use super::{NbError, ExtractionResult, ExtractionMode, QualityMetrics};

#[derive(Debug, Clone, Copy)]
pub enum MindtctMode {
    /// m1: standard minutiae detection
    M1,
    /// m1 + browsable output (human-inspectable, slower)
    M1PlusBrws,
    /// iso: ISO 19794-2 compliant output
    Iso,
}

pub fn extract(
    image: &Path,
    out_base: &str,
    nbis_home: &Path,
    mode: MindtctMode,
) -> Result<ExtractionResult, NbError> {
    let bin = nbis_home.join("bin/mindtct");
    if !bin.exists() {
        return Err(NbError::BinaryNotFound(bin.display().to_string()));
    }
    let flags = match mode {
        MindtctMode::M1 => vec!["-m1"],
        MindtctMode::M1PlusBrws => vec!["-m1", "-b"],
        MindtctMode::Iso => vec!["-m1", "-i"],  // ISO output
    };

    let out = Command::new(&bin)
        .args(&flags)
        .arg(image)
        .arg(out_base)
        .output()?;

    if !out.status.success() {
        return Err(NbError::ToolFailed {
            tool: "mindtct".into(),
            exit_code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    // mindtct produces: <base>.xyt (minutiae), <base>.brw (browsable), <base>.dfm (map)
    let xyt = PathBuf::from(format!("{out_base}.xyt"));
    let xyt_path = image.parent()
        .map(|p| p.join(&xyt))
        .unwrap_or(xyt);

    if !xyt_path.exists() {
        return Err(NbError::TemplateNotFound(xyt_path.display().to_string()));
    }

    // Parse .xyt for minutiae count
    let xyt_data = std::fs::read_to_string(&xyt_path)?;
    let minutiae_count = xyt_data.lines().filter(|l| !l.trim().is_empty()).count();

    Ok(ExtractionResult {
        template_path: xyt_path,
        mode: ExtractionMode::NbisXyt,
        minutiae_count,
        quality: parse_quality_from_xyt(&xyt_data),
    })
}

fn parse_quality_from_xyt(xyt_data: &str) -> QualityMetrics {
    // .xyt format: x y theta quality
    // quality is 0-100 per minutia
    let mut total = 0u64;
    let mut min = u32::MAX;
    let mut max = 0u32;
    let mut n = 0u64;
    for line in xyt_data.lines() {
        if line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }
        if let Ok(q) = parts[3].parse::<u32>() {
            total += q as u64;
            min = min.min(q);
            max = max.max(q);
            n += 1;
        }
    }
    let mean = if n > 0 { (total / n) as u32 } else { 0 };
    QualityMetrics {
        mean_minutiae_quality: mean,
        min_minutiae_quality: if min == u32::MAX { 0 } else { min },
        max_minutiae_quality: max,
    }
}
