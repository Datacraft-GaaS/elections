use crate::domain::VeinTemplate;

pub const DEFAULT_THRESHOLD: f32 = 0.60;

pub struct Score {
    pub raw: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
    pub best_rotation_deg: f32,
    pub best_tx: i16,
    pub best_ty: i16,
}

const ROTATIONS: [f32; 7] = [-15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 15.0];
const TX_STEPS: [i16; 5] = [-20, -10, 0, 10, 20];
const TY_STEPS: [i16; 5] = [-20, -10, 0, 10, 20];

pub fn match_vein(probe: &VeinTemplate, gallery: &VeinTemplate) -> Score {
    let pw = probe.width as usize;
    let ph = probe.height as usize;
    let gw = gallery.width as usize;
    let gh = gallery.height as usize;

    let mut best_score: f32 = -1.0;
    let mut best_rot = 0.0f32;
    let mut best_tx = 0i16;
    let mut best_ty = 0i16;

    for &rot_deg in &ROTATIONS {
        let angle_rad = rot_deg * std::f32::consts::PI / 180.0;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        for &tx in &TX_STEPS {
            for &ty in &TY_STEPS {
                let shifted = rotate_translate(
                    &gallery.pattern_bitmap,
                    gw,
                    gh,
                    cos_a,
                    sin_a,
                    tx,
                    ty,
                    pw,
                    ph,
                );
                let score = ncc(&probe.pattern_bitmap, &shifted);
                if score > best_score {
                    best_score = score;
                    best_rot = rot_deg;
                    best_tx = tx;
                    best_ty = ty;
                }
            }
        }
    }

    let raw = best_score.clamp(0.0, 1.0);
    Score {
        raw,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
        best_rotation_deg: best_rot,
        best_tx,
        best_ty,
    }
}

/// Rotate and translate gallery bitmap, sampling into a probe-sized output.
fn rotate_translate(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    cos_a: f32,
    sin_a: f32,
    tx: i16,
    ty: i16,
    dst_w: usize,
    dst_h: usize,
) -> Vec<u8> {
    let cx_src = src_w as f32 / 2.0;
    let cy_src = src_h as f32 / 2.0;
    let cx_dst = dst_w as f32 / 2.0;
    let cy_dst = dst_h as f32 / 2.0;

    let mut dst = vec![0u8; dst_w * dst_h];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let rx = dx as f32 - cx_dst - tx as f32;
            let ry = dy as f32 - cy_dst - ty as f32;
            // Inverse rotation (rotate back to find source pixel).
            let sx = cos_a * rx + sin_a * ry + cx_src;
            let sy = -sin_a * rx + cos_a * ry + cy_src;
            let six = sx.round() as i32;
            let siy = sy.round() as i32;
            if six >= 0 && six < src_w as i32 && siy >= 0 && siy < src_h as i32 {
                dst[dy * dst_w + dx] = src[siy as usize * src_w + six as usize];
            }
        }
    }
    dst
}

pub fn ncc(a: &[u8], b: &[u8]) -> f32 {
    let n = a.len().min(b.len()) as f64;
    if n == 0.0 {
        return 0.0;
    }

    let mean_a: f64 = a.iter().map(|&v| v as f64).sum::<f64>() / n;
    let mean_b: f64 = b.iter().map(|&v| v as f64).sum::<f64>() / n;

    let mut num = 0.0f64;
    let mut sq_a = 0.0f64;
    let mut sq_b = 0.0f64;

    for (&av, &bv) in a.iter().zip(b.iter()) {
        let da = av as f64 - mean_a;
        let db = bv as f64 - mean_b;
        num += da * db;
        sq_a += da * da;
        sq_b += db * db;
    }

    let denom = (sq_a * sq_b).sqrt();
    if denom < 1e-12 {
        return 0.0;
    }

    (num / denom) as f32
}
