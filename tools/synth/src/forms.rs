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

// Form 34A image generator.
//
// Determinism note: all pixel operations use seeded ChaCha20Rng and
// fixed-layout bitmap font. Output is stable within a build on any arch.
// Cross-arch PNG byte identity depends on the `image` crate's zlib encoder
// producing identical deflate output, which holds for the pure-Rust miniz_oxide
// backend used by default. Wave 5 reproducibility work will add a golden-hash
// CI check per target triple if cross-arch drift is ever observed.

use anyhow::Result;
use image::{ImageBuffer, Rgb, RgbImage};
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use std::io::Write;
use std::path::Path;

use crate::rng::{child_seed, seeded};
use crate::DegradationFlags;

// ---------------------------------------------------------------------------
// Bitmap font: 5-wide x 8-tall pixels per glyph.
// Each row is a u8 where bit 4 is leftmost pixel.
// ---------------------------------------------------------------------------

const GLYPH_W: u32 = 5;
const GLYPH_H: u32 = 8;

#[rustfmt::skip]
const DIGITS: [[u8; 8]; 10] = [
    // 0
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110, 0b00000],
    // 1
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0b00000],
    // 2
    [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111, 0b00000],
    // 3
    [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110, 0b00000],
    // 4
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010, 0b00000],
    // 5
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110, 0b00000],
    // 6
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110, 0b00000],
    // 7
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00000],
    // 8
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110, 0b00000],
    // 9
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100, 0b00000],
];

fn draw_digit(img: &mut RgbImage, x0: u32, y0: u32, digit: u8, color: Rgb<u8>, scale: u32) {
    let d = (digit % 10) as usize;
    let bitmap = &DIGITS[d];
    for (row, &bits) in bitmap.iter().enumerate() {
        for col in 0..GLYPH_W {
            if bits & (0b10000 >> col) != 0 {
                let px = x0 + col * scale;
                let py = y0 + row as u32 * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        let nx = px + dx;
                        let ny = py + dy;
                        if nx < img.width() && ny < img.height() {
                            img.put_pixel(nx, ny, color);
                        }
                    }
                }
            }
        }
    }
}

fn draw_number(img: &mut RgbImage, x0: u32, y0: u32, n: u64, color: Rgb<u8>, scale: u32) {
    let s = n.to_string();
    let stride = (GLYPH_W + 1) * scale;
    for (i, ch) in s.chars().enumerate() {
        let d = ch as u8 - b'0';
        draw_digit(img, x0 + i as u32 * stride, y0, d, color, scale);
    }
}

fn draw_label_pixels(img: &mut RgbImage, x0: u32, y0: u32, text: &str, scale: u32) {
    // Draw text as tiny dark dots using an approximation for A-Z letters.
    // For the synth tool this is sufficient to make forms human-readable.
    let ink = Rgb([30u8, 30, 30]);
    for (i, ch) in text.chars().take(32).enumerate() {
        let x = x0 + i as u32 * (GLYPH_W + 1) * scale;
        if let Some(d) = ch.to_digit(10) {
            draw_digit(img, x, y0, d as u8, ink, scale);
        } else {
            // Draw a small dash for non-digit characters.
            let mid_y = y0 + GLYPH_H * scale / 2;
            for dx in 0..GLYPH_W * scale {
                if x + dx < img.width() && mid_y < img.height() {
                    img.put_pixel(x + dx, mid_y, ink);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Candidate row: draw candidate name slots + vote count box.
// ---------------------------------------------------------------------------

fn draw_candidate_row(img: &mut RgbImage, y: u32, candidate_num: u8, votes: u64, scale: u32) {
    let ink = Rgb([20u8, 20, 80]);
    let margin = 20u32;

    // Candidate number.
    draw_digit(img, margin, y, candidate_num, ink, scale);

    // Votes box.
    let vx = margin + (GLYPH_W + 1) * scale * 6;
    draw_number(img, vx, y, votes, ink, scale);
}

// ---------------------------------------------------------------------------
// Degradation effects
// ---------------------------------------------------------------------------

fn apply_torn(img: &mut RgbImage, rng: &mut ChaCha20Rng) {
    let w = img.width();
    let h = img.height();
    // Simulate a tear along the right edge.
    let tear_x_start = (w as f64 * 0.88) as u32;
    for y in 0..h {
        let offset: i32 = rng.gen_range(-3i32..4);
        let tx = (tear_x_start as i32 + (y as i32 / 20) * offset)
            .clamp(tear_x_start as i32, w as i32 - 1) as u32;
        for x in tx..w {
            img.put_pixel(x, y, Rgb([240u8, 235, 220]));
        }
    }
    // A few random black tear-edge pixels.
    for _ in 0..300 {
        let tx = rng.gen_range(tear_x_start.saturating_sub(3)..tear_x_start + 3);
        let ty = rng.gen_range(0..h);
        if tx < w {
            img.put_pixel(tx, ty, Rgb([10u8, 10, 10]));
        }
    }
}

fn apply_wet(img: &mut RgbImage, rng: &mut ChaCha20Rng) {
    let w = img.width();
    let h = img.height();
    // Add 5-10 dark blotches.
    let blotch_count = rng.gen_range(5u32..11);
    for _ in 0..blotch_count {
        let cx = rng.gen_range(0..w);
        let cy = rng.gen_range(0..h);
        let radius = rng.gen_range(5u32..25);
        let alpha: u8 = rng.gen_range(30u8..90);
        for dy in 0..radius * 2 {
            for dx in 0..radius * 2 {
                let px = cx.saturating_sub(radius) + dx;
                let py = cy.saturating_sub(radius) + dy;
                if px < w && py < h {
                    let dist_sq =
                        (dx as i32 - radius as i32).pow(2) + (dy as i32 - radius as i32).pow(2);
                    if dist_sq <= (radius as i32).pow(2) {
                        let p = img.get_pixel(px, py);
                        let blended = Rgb([
                            p[0].saturating_sub(alpha / 3),
                            p[1].saturating_sub(alpha / 4),
                            p[2].saturating_sub(alpha / 5),
                        ]);
                        img.put_pixel(px, py, blended);
                    }
                }
            }
        }
    }
}

fn apply_low_light(img: &mut RgbImage, factor: f32) {
    for p in img.pixels_mut() {
        p[0] = (p[0] as f32 * factor) as u8;
        p[1] = (p[1] as f32 * factor) as u8;
        p[2] = (p[2] as f32 * factor) as u8;
    }
}

fn apply_angled(img: RgbImage, rng: &mut ChaCha20Rng) -> RgbImage {
    // Rotate by a small deterministic angle (between -8 and +8 degrees).
    let angle_deg: f64 = rng.gen_range(-8.0f64..8.0f64);
    let angle_rad = angle_deg.to_radians();
    let w = img.width();
    let h = img.height();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let white = Rgb([255u8, 255, 255]);

    let mut out = ImageBuffer::from_pixel(w, h, white);
    for y in 0..h {
        for x in 0..w {
            // Inverse rotation: find source pixel.
            let rx = (x as f64 - cx) * cos_a + (y as f64 - cy) * sin_a + cx;
            let ry = -(x as f64 - cx) * sin_a + (y as f64 - cy) * cos_a + cy;
            let sx = rx.round() as i64;
            let sy = ry.round() as i64;
            if sx >= 0 && sy >= 0 && sx < w as i64 && sy < h as i64 {
                out.put_pixel(x, y, *img.get_pixel(sx as u32, sy as u32));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Form record
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct FormRecord {
    pub form_id: String,
    pub station_code: String,
    pub candidate_count: u8,
    pub total_votes: u64,
}

pub fn generate(
    seed: u64,
    count: u64,
    stations: &[crate::geo::PollingStation],
    degradation: &DegradationFlags,
    out_dir: &Path,
) -> Result<Vec<FormRecord>> {
    assert!(!stations.is_empty());

    let mut rng: ChaCha20Rng = seeded(child_seed(seed, "forms"));
    std::fs::create_dir_all(out_dir)?;

    let mut records = Vec::with_capacity(count as usize);

    for i in 0..count {
        let station = &stations[rng.gen_range(0..stations.len())];
        let candidate_count: u8 = rng.gen_range(2u8..9);
        let votes_per_candidate: Vec<u64> = (0..candidate_count)
            .map(|_| rng.gen_range(0u64..500))
            .collect();
        let total_votes: u64 = votes_per_candidate.iter().sum();

        // Render form image.
        let img = render_form(
            station,
            &votes_per_candidate,
            total_votes,
            i,
            &mut rng,
            degradation,
        );

        let fname = format!("form34a_{:06}.png", i + 1);
        let fpath = out_dir.join(&fname);
        img.save(&fpath)?;

        records.push(FormRecord {
            form_id: format!("F34A-{:07}", i + 1),
            station_code: station.code.clone(),
            candidate_count,
            total_votes,
        });
    }

    Ok(records)
}

fn render_form(
    station: &crate::geo::PollingStation,
    votes: &[u64],
    total: u64,
    form_idx: u64,
    rng: &mut ChaCha20Rng,
    deg: &DegradationFlags,
) -> RgbImage {
    let width = 640u32;
    let height = 480u32;
    let white = Rgb([255u8, 255, 255]);
    let dark = Rgb([30u8, 30, 30]);
    let scale = 2u32;

    let mut img: RgbImage = ImageBuffer::from_pixel(width, height, white);

    // Draw border.
    for x in 0..width {
        img.put_pixel(x, 0, dark);
        img.put_pixel(x, height - 1, dark);
    }
    for y in 0..height {
        img.put_pixel(0, y, dark);
        img.put_pixel(width - 1, y, dark);
    }

    // Header: form number.
    draw_label_pixels(&mut img, 20, 10, "FORM34A", scale);
    draw_number(&mut img, 200, 10, form_idx + 1, dark, scale);

    // Station code line.
    draw_label_pixels(&mut img, 20, 30, "STATION", scale);
    draw_label_pixels(&mut img, 120, 30, &station.code, scale);

    // County.
    draw_label_pixels(&mut img, 20, 50, "COUNTY", scale);
    draw_number(&mut img, 120, 50, station.county_code as u64, dark, scale);

    // Candidates.
    let mut y = 80u32;
    for (idx, &v) in votes.iter().enumerate() {
        draw_candidate_row(&mut img, y, (idx + 1) as u8, v, scale);
        y += (GLYPH_H + 2) * scale;
    }

    // Total votes.
    y += 4;
    draw_label_pixels(&mut img, 20, y, "TOTAL", scale);
    draw_number(&mut img, 120, y, total, dark, scale);

    // Apply degradation.
    if deg.wet {
        apply_wet(&mut img, rng);
    }
    if deg.low_light {
        apply_low_light(&mut img, 0.45);
    }
    if deg.torn {
        apply_torn(&mut img, rng);
    }
    if deg.angled {
        apply_angled(img, rng)
    } else {
        img
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_digit_does_not_panic() {
        let mut img: RgbImage = ImageBuffer::from_pixel(100, 100, Rgb([255u8, 255, 255]));
        draw_digit(&mut img, 5, 5, 7, Rgb([0u8, 0, 0]), 2);
    }

    #[test]
    fn degradation_flags_parse() {
        let f: DegradationFlags = "torn,wet".parse().unwrap();
        assert!(f.torn);
        assert!(f.wet);
        assert!(!f.angled);
    }
}
