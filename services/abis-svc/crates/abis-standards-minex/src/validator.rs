use crate::template::{MinexTemplate, IMAGE_HEIGHT, IMAGE_WIDTH, MAX_MINUTIAE};

#[derive(Debug, Clone)]
pub struct ConformanceResult {
    pub passes: bool,
    pub violations: Vec<String>,
    pub minutiae_count: usize,
    pub estimated_quality: f32,
}

pub struct MinexValidator;

impl MinexValidator {
    pub fn validate(template: &MinexTemplate) -> ConformanceResult {
        let mut violations = Vec::new();

        if template.minutiae.len() > MAX_MINUTIAE {
            violations.push(format!(
                "minutiae count {} exceeds maximum {}",
                template.minutiae.len(),
                MAX_MINUTIAE
            ));
        }

        if template.minutiae.is_empty() {
            violations.push("template contains no minutiae".into());
        }

        for (i, m) in template.minutiae.iter().enumerate() {
            if m.x >= IMAGE_WIDTH || m.y >= IMAGE_HEIGHT {
                violations.push(format!(
                    "minutia[{i}] coordinate ({},{}) out of bounds",
                    m.x, m.y
                ));
            }
            if m.quality > 100 {
                violations.push(format!("minutia[{i}] quality {} exceeds 100", m.quality));
            }
        }

        let fp = template.finger_position;
        if !(1..=10).contains(&fp) {
            violations.push(format!("finger_position {fp} not in valid range 1-10"));
        }

        let count = template.minutiae.len();
        let estimated_quality = if count == 0 {
            0.0
        } else {
            template
                .minutiae
                .iter()
                .map(|m| m.quality as f32)
                .sum::<f32>()
                / count as f32
        };

        ConformanceResult {
            passes: violations.is_empty(),
            violations,
            minutiae_count: count,
            estimated_quality,
        }
    }
}
