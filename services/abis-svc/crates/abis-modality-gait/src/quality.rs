use crate::domain::GaitTemplate;

#[derive(Debug, Clone)]
pub struct GaitQuality {
    pub cycle_count_score: f32,
    pub speed_score: f32,
    pub overall: f32,
}

pub fn score_quality(template: &GaitTemplate) -> GaitQuality {
    let n = template.cycles.len();

    let cycle_count_score = if n == 0 {
        0.0
    } else if n >= 3 && n <= 8 {
        100.0
    } else if n < 3 {
        (n as f32 / 3.0) * 100.0
    } else {
        // More than 8: mild penalty, still useful.
        let excess = (n - 8) as f32;
        (100.0 - excess * 5.0).max(0.0)
    };

    let speed = template.walking_speed_ms;
    let speed_score = if speed >= 1.0 && speed <= 2.0 {
        100.0
    } else if speed < 1.0 {
        (speed / 1.0) * 100.0
    } else {
        // Above 2.0 m/s.
        let excess = speed - 2.0;
        (100.0 - excess * 50.0).max(0.0)
    };

    let overall = if n == 0 {
        0.0
    } else {
        (cycle_count_score + speed_score) / 2.0
    };

    GaitQuality {
        cycle_count_score,
        speed_score,
        overall,
    }
}
