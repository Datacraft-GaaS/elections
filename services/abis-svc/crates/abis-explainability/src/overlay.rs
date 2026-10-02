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

use crate::ExplainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutiaPoint {
    pub x: f32,
    pub y: f32,
    pub angle_deg: f32,
    pub quality: f32,
    pub is_matched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutiaePair {
    pub probe_idx: usize,
    pub gallery_idx: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

pub fn generate_svg(
    probe_minutiae: &[MinutiaPoint],
    gallery_minutiae: &[MinutiaPoint],
    pairs: &[MinutiaePair],
    config: &OverlayConfig,
) -> Result<String, ExplainError> {
    if probe_minutiae.is_empty() {
        return Err(ExplainError::EmptyTemplate);
    }

    let w = config.width;
    let h = config.height;
    let total_w = w * 2;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w} {h}" width="{total_w}" height="{h}">"#
    );

    let probe_matched: std::collections::HashSet<usize> =
        pairs.iter().map(|p| p.probe_idx).collect();
    let gallery_matched: std::collections::HashSet<usize> =
        pairs.iter().map(|p| p.gallery_idx).collect();

    for (i, m) in probe_minutiae.iter().enumerate() {
        let color = if probe_matched.contains(&i) {
            "green"
        } else {
            "red"
        };
        let cx = m.x * config.scale;
        let cy = m.y * config.scale;
        svg.push_str(&format!(
            r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="4" fill="{color}" opacity="0.8"/>"#
        ));
        let angle_rad = m.angle_deg.to_radians();
        let ex = cx + 8.0 * angle_rad.cos();
        let ey = cy + 8.0 * angle_rad.sin();
        svg.push_str(&format!(
            r#"<line x1="{cx:.1}" y1="{cy:.1}" x2="{ex:.1}" y2="{ey:.1}" stroke="{color}" stroke-width="1"/>"#
        ));
    }

    for (i, m) in gallery_minutiae.iter().enumerate() {
        let color = if gallery_matched.contains(&i) {
            "green"
        } else {
            "blue"
        };
        let cx = w as f32 + m.x * config.scale;
        let cy = m.y * config.scale;
        svg.push_str(&format!(
            r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="4" fill="{color}" opacity="0.8"/>"#
        ));
        let angle_rad = m.angle_deg.to_radians();
        let ex = cx + 8.0 * angle_rad.cos();
        let ey = cy + 8.0 * angle_rad.sin();
        svg.push_str(&format!(
            r#"<line x1="{cx:.1}" y1="{cy:.1}" x2="{ex:.1}" y2="{ey:.1}" stroke="{color}" stroke-width="1"/>"#
        ));
    }

    for pair in pairs {
        if pair.probe_idx < probe_minutiae.len() && pair.gallery_idx < gallery_minutiae.len() {
            let pm = &probe_minutiae[pair.probe_idx];
            let gm = &gallery_minutiae[pair.gallery_idx];
            let x1 = pm.x * config.scale;
            let y1 = pm.y * config.scale;
            let x2 = w as f32 + gm.x * config.scale;
            let y2 = gm.y * config.scale;
            let opacity = pair.confidence.clamp(0.0, 1.0);
            svg.push_str(&format!(
                r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="green" stroke-width="1.5" opacity="{opacity:.2}" data-confidence="{opacity:.2}"/>"#
            ));
        }
    }

    svg.push_str("</svg>");
    Ok(svg)
}
