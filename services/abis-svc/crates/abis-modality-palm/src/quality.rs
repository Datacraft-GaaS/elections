//! Palm quality scoring with anatomical zone weighting.

use crate::domain::{PalmTemplate, PalmZone, PalmZoneId};

/// Zone weight table (must sum to 1.0).
const ZONE_WEIGHTS: &[(PalmZoneId, f32)] = &[
    (PalmZoneId::Writer, 0.30),
    (PalmZoneId::Upper, 0.20),
    (PalmZoneId::Lower, 0.20),
    (PalmZoneId::Thenar, 0.15),
    (PalmZoneId::Hypothenar, 0.15),
];

/// Optimal minutiae count per zone for full quality.
const OPTIMAL_MINUTIAE_PER_ZONE: f32 = 20.0;

/// Compute a weighted quality score in [0.0, 100.0] from a palm template.
pub fn score_quality(template: &PalmTemplate) -> f32 {
    if template.minutiae.is_empty() {
        return 0.0;
    }

    let zones = segment_zones(template);
    let mut weighted_sum = 0.0_f32;

    for (zone_id, weight) in ZONE_WEIGHTS {
        let zone_minutiae = zones
            .iter()
            .find(|z| &z.zone == zone_id)
            .map_or(0, |z| z.minutiae.len());
        let zone_score = (zone_minutiae as f32 / OPTIMAL_MINUTIAE_PER_ZONE).min(1.0);
        weighted_sum += zone_score * weight;
    }

    (weighted_sum * 100.0).min(100.0)
}

/// Segment the template minutiae into anatomical palm zones.
///
/// Partitioning uses simple y-axis thirds for Upper/Lower/Writer and x-axis
/// halves for Thenar/Hypothenar in the lower band.
pub fn segment_zones(template: &PalmTemplate) -> Vec<PalmZone> {
    // Compute bounding box.
    let (max_x, max_y) = template
        .minutiae
        .iter()
        .fold((1u16, 1u16), |(mx, my), m| (mx.max(m.x), my.max(m.y)));

    let y_third = max_y / 3;
    let x_half = max_x / 2;

    let mut writer = Vec::new();
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    let mut thenar = Vec::new();
    let mut hypothenar = Vec::new();

    for m in &template.minutiae {
        if m.y < y_third {
            upper.push(m.clone());
        } else if m.y < 2 * y_third {
            lower.push(m.clone());
        } else if m.x < x_half {
            thenar.push(m.clone());
        } else if m.x >= x_half && m.y >= 2 * y_third {
            hypothenar.push(m.clone());
        } else {
            writer.push(m.clone());
        }
    }

    // Ensure all 5 zones are present even if empty.
    vec![
        PalmZone {
            zone: PalmZoneId::Writer,
            minutiae: writer,
        },
        PalmZone {
            zone: PalmZoneId::Upper,
            minutiae: upper,
        },
        PalmZone {
            zone: PalmZoneId::Lower,
            minutiae: lower,
        },
        PalmZone {
            zone: PalmZoneId::Thenar,
            minutiae: thenar,
        },
        PalmZone {
            zone: PalmZoneId::Hypothenar,
            minutiae: hypothenar,
        },
    ]
}
