//! Non-maximum suppression (Greedy NMS by confidence).
//!
//! Reference: Hosang, J. et al. (2017). "Learning Non-Maximum Suppression."
//! CVPR 2017. We use the classical greedy variant here.

use crate::FaceBox;

/// Greedy NMS: keep the highest-confidence box; suppress any box whose IoU
/// with the kept box exceeds `iou_threshold`.
///
/// Input boxes do NOT need to be pre-sorted; this function sorts by confidence.
pub fn nms(mut boxes: Vec<FaceBox>, iou_threshold: f32) -> Vec<FaceBox> {
    // Sort descending by confidence.
    boxes.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut kept: Vec<FaceBox> = Vec::new();
    let mut suppressed = vec![false; boxes.len()];

    for i in 0..boxes.len() {
        if suppressed[i] {
            continue;
        }
        kept.push(boxes[i].clone());
        for j in (i + 1)..boxes.len() {
            if suppressed[j] {
                continue;
            }
            if boxes[i].bbox.iou(&boxes[j].bbox) > iou_threshold {
                suppressed[j] = true;
            }
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoundingBox, FaceBox};

    fn fb(x: u32, y: u32, w: u32, h: u32, conf: f32) -> FaceBox {
        FaceBox::new(BoundingBox::new(x, y, w, h), conf)
    }

    #[test]
    fn nms_keeps_highest_confidence() {
        let boxes = vec![fb(0, 0, 10, 10, 0.9), fb(1, 1, 10, 10, 0.7)];
        let result = nms(boxes, 0.5);
        assert_eq!(result.len(), 1);
        assert!((result[0].confidence - 0.9).abs() < 1e-5);
    }

    #[test]
    fn nms_keeps_non_overlapping() {
        let boxes = vec![fb(0, 0, 10, 10, 0.9), fb(100, 100, 10, 10, 0.8)];
        let result = nms(boxes, 0.5);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn nms_empty_input() {
        let result = nms(vec![], 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn nms_single_box() {
        let result = nms(vec![fb(0, 0, 20, 20, 0.95)], 0.5);
        assert_eq!(result.len(), 1);
    }
}
