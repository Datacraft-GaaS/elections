//! IoU + embedding similarity tracker (SORT-style greedy assignment).
//!
//! Reference: Bewley et al. (2016). "Simple Online and Realtime Tracking." ICIP.

use crate::track::{FrameDetection, Track, TrackId, TrackUpdate};

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot.clamp(-1.0, 1.0)
}

/// Multi-object face tracker.
///
/// Assigns detections to existing tracks via a combined IoU + embedding
/// similarity score. Tracks that exceed `max_missed` frames without a match
/// are pruned.
pub struct Tracker {
    tracks: Vec<Track>,
    next_id: TrackId,
    /// Minimum IoU for a bbox match.
    iou_thresh: f32,
    #[allow(dead_code)]
    emb_thresh: f32,
    /// Frames a track may go undetected before being dropped.
    max_missed: u32,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            next_id: 1,
            iou_thresh: 0.3,
            emb_thresh: 0.5,
            max_missed: 5,
        }
    }
}

impl Tracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thresholds(iou_thresh: f32, emb_thresh: f32, max_missed: u32) -> Self {
        Self {
            iou_thresh,
            emb_thresh,
            max_missed,
            ..Self::default()
        }
    }

    /// Process one frame of detections, returning track update events.
    pub fn feed_frame(&mut self, detections: Vec<FrameDetection>) -> Vec<TrackUpdate> {
        let mut updates: Vec<TrackUpdate> = Vec::new();
        let mut matched_track_ids: Vec<TrackId> = Vec::new();
        let mut unmatched_dets: Vec<FrameDetection> = Vec::new();

        // Greedy assignment: for each detection, find the best-matching active track.
        'det_loop: for det in &detections {
            let mut best_score = -1.0f32;
            let mut best_track_idx: Option<usize> = None;

            for (ti, track) in self.tracks.iter().enumerate() {
                if matched_track_ids.contains(&track.id) {
                    continue;
                }

                let iou = track.last_bbox().map(|bb| bb.iou(&det.bbox)).unwrap_or(0.0);

                let emb_score = match (track.last_embedding(), det.embedding.as_deref()) {
                    (Some(te), Some(de)) => cosine_sim(te, de),
                    _ => 0.5, // neutral when one or both embeddings absent
                };

                // Combined score: 0.6 * IoU + 0.4 * embedding similarity.
                let score = 0.6 * iou + 0.4 * emb_score;

                if iou >= self.iou_thresh && score > best_score {
                    best_score = score;
                    best_track_idx = Some(ti);
                }
            }

            if let Some(ti) = best_track_idx {
                let track_id = self.tracks[ti].id;
                matched_track_ids.push(track_id);
                let det_clone = det.clone();
                self.tracks[ti].add_detection(det.clone());
                updates.push(TrackUpdate {
                    track_id,
                    detection: det_clone,
                    is_new: false,
                });
                continue 'det_loop;
            }

            unmatched_dets.push(det.clone());
        }

        // Increment missed counter for unmatched tracks.
        for track in self.tracks.iter_mut() {
            if !matched_track_ids.contains(&track.id) {
                track.missed_frames += 1;
            }
        }

        // Start new tracks for unmatched detections.
        for det in unmatched_dets {
            let id = self.next_id;
            self.next_id += 1;
            let det_clone = det.clone();
            self.tracks.push(Track::new(id, det));
            updates.push(TrackUpdate {
                track_id: id,
                detection: det_clone,
                is_new: true,
            });
        }

        // Prune dead tracks.
        self.tracks.retain(|t| t.missed_frames <= self.max_missed);

        updates
    }

    pub fn active_tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}
