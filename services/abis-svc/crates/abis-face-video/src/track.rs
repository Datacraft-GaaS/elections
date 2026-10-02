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

//! Track and per-frame detection types.

use serde::{Deserialize, Serialize};

/// Unique track identifier.
pub type TrackId = u64;

/// Bounding box in pixel coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl BBox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn iou(&self, other: &Self) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);
        if x2 <= x1 || y2 <= y1 {
            return 0.0;
        }
        let inter = (x2 - x1) * (y2 - y1);
        let union = self.w * self.h + other.w * other.h - inter;
        if union < 1e-6 {
            return 0.0;
        }
        (inter / union).clamp(0.0, 1.0)
    }
}

/// A single face detection in one video frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameDetection {
    /// Bounding box.
    pub bbox: BBox,
    /// Detection confidence in [0.0, 1.0].
    pub confidence: f32,
    /// L2-normalised 512-d embedding (may be None if embedder not run yet).
    pub embedding: Option<Vec<f32>>,
    /// Quality score [0.0, 1.0] from face quality assessment.
    pub quality: f32,
    /// Frame timestamp (milliseconds from video start).
    pub timestamp_ms: u64,
}

impl FrameDetection {
    pub fn new(bbox: BBox, confidence: f32, quality: f32, timestamp_ms: u64) -> Self {
        Self {
            bbox,
            confidence,
            embedding: None,
            quality,
            timestamp_ms,
        }
    }

    pub fn with_embedding(mut self, emb: Vec<f32>) -> Self {
        self.embedding = Some(emb);
        self
    }
}

/// A sequence of detections associated with the same identity across frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub detections: Vec<FrameDetection>,
    /// Number of frames since the track was last updated (for pruning).
    pub missed_frames: u32,
}

impl Track {
    pub fn new(id: TrackId, first: FrameDetection) -> Self {
        Self {
            id,
            detections: vec![first],
            missed_frames: 0,
        }
    }

    pub fn add_detection(&mut self, det: FrameDetection) {
        self.missed_frames = 0;
        self.detections.push(det);
    }

    pub fn last_bbox(&self) -> Option<&BBox> {
        self.detections.last().map(|d| &d.bbox)
    }

    pub fn last_embedding(&self) -> Option<&Vec<f32>> {
        self.detections
            .iter()
            .rev()
            .find_map(|d| d.embedding.as_ref())
    }

    pub fn frame_count(&self) -> usize {
        self.detections.len()
    }
}

/// Notification returned by the Tracker after processing a frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackUpdate {
    pub track_id: TrackId,
    pub detection: FrameDetection,
    pub is_new: bool,
}
