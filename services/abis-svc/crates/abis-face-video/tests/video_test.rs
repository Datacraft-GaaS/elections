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

use eel_abis_face_video::{
    fusion::fuse_track,
    track::{BBox, FrameDetection},
    tracker::Tracker,
    Track,
};

fn unit_emb(val: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 512];
    v[0] = val;
    v[1] = (1.0 - val * val).max(0.0).sqrt();
    v
}

fn det(x: f32, y: f32, quality: f32, ts: u64) -> FrameDetection {
    FrameDetection::new(BBox::new(x, y, 50.0, 50.0), 0.9, quality, ts)
}

fn det_with_emb(x: f32, y: f32, quality: f32, ts: u64, emb: Vec<f32>) -> FrameDetection {
    det(x, y, quality, ts).with_embedding(emb)
}

// --- BBox tests ---

#[test]
fn bbox_iou_identical() {
    let b = BBox::new(0.0, 0.0, 10.0, 10.0);
    assert!((b.iou(&b) - 1.0).abs() < 1e-5);
}

#[test]
fn bbox_iou_no_overlap() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(50.0, 50.0, 10.0, 10.0);
    assert!(a.iou(&b) < 1e-5);
}

// --- Tracker tests ---

#[test]
fn tracker_new_detection_starts_track() {
    let mut t = Tracker::new();
    let updates = t.feed_frame(vec![det(0.0, 0.0, 0.8, 0)]);
    assert_eq!(updates.len(), 1);
    assert!(updates[0].is_new);
}

#[test]
fn tracker_continues_overlapping_track() {
    let mut t = Tracker::new();
    t.feed_frame(vec![det(0.0, 0.0, 0.8, 0)]);
    // Slightly shifted detection in next frame should continue the same track.
    let updates = t.feed_frame(vec![det(5.0, 5.0, 0.8, 33)]);
    assert_eq!(updates.len(), 1);
    assert!(!updates[0].is_new, "should continue existing track");
    assert_eq!(updates[0].track_id, 1);
}

#[test]
fn tracker_distant_detection_starts_new_track() {
    let mut t = Tracker::new();
    t.feed_frame(vec![det(0.0, 0.0, 0.8, 0)]);
    // Detection far away from existing track => new track.
    let updates = t.feed_frame(vec![det(500.0, 500.0, 0.8, 33)]);
    assert_eq!(updates.len(), 1);
    assert!(updates[0].is_new);
}

#[test]
fn tracker_two_simultaneous_faces() {
    let mut t = Tracker::new();
    let updates = t.feed_frame(vec![det(0.0, 0.0, 0.8, 0), det(300.0, 0.0, 0.8, 0)]);
    assert_eq!(updates.len(), 2);
    assert_ne!(updates[0].track_id, updates[1].track_id);
}

#[test]
fn tracker_prunes_missed_tracks() {
    let mut t = Tracker::with_thresholds(0.3, 0.5, 2);
    t.feed_frame(vec![det(0.0, 0.0, 0.8, 0)]);
    // Feed 3 frames with no detections.
    t.feed_frame(vec![]);
    t.feed_frame(vec![]);
    t.feed_frame(vec![]);
    assert_eq!(
        t.track_count(),
        0,
        "track should be pruned after max_missed frames"
    );
}

#[test]
fn tracker_identity_preserved_with_embeddings() {
    let mut t = Tracker::new();
    let emb1 = unit_emb(0.9);
    let emb2 = unit_emb(0.85); // similar embedding
    t.feed_frame(vec![det_with_emb(0.0, 0.0, 0.9, 0, emb1)]);
    let updates = t.feed_frame(vec![det_with_emb(5.0, 5.0, 0.9, 33, emb2)]);
    assert_eq!(updates.len(), 1);
    assert!(!updates[0].is_new);
}

#[test]
fn tracker_active_tracks_reflects_state() {
    let mut t = Tracker::new();
    t.feed_frame(vec![det(0.0, 0.0, 0.8, 0)]);
    assert_eq!(t.active_tracks().len(), 1);
}

// --- Fusion tests ---

#[test]
fn fuse_empty_track_returns_no_embedding() {
    let track = Track::new(1, det(0.0, 0.0, 0.8, 0)); // no embeddings
    let fused = fuse_track(&track);
    assert!(fused.embedding.is_none());
}

#[test]
fn fuse_single_frame_returns_embedding() {
    let emb = unit_emb(0.6);
    let d = det_with_emb(0.0, 0.0, 0.8, 0, emb.clone());
    let track = Track::new(1, d);
    let fused = fuse_track(&track);
    assert!(fused.embedding.is_some());
    assert_eq!(fused.frame_count, 1);
}

#[test]
fn fuse_result_is_l2_normalised() {
    let emb = unit_emb(0.7);
    let mut track = Track::new(1, det_with_emb(0.0, 0.0, 0.9, 0, emb.clone()));
    track.add_detection(det_with_emb(5.0, 5.0, 0.8, 33, emb));
    let fused = fuse_track(&track);
    let v = fused.embedding.unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={}", norm);
}

#[test]
fn fuse_low_quality_frames_contribute_less() {
    // Two frames: one high quality with emb pointing in direction A,
    // one low quality with emb pointing in direction B.
    // Fused result should be closer to A.
    let mut emb_a = vec![0.0f32; 512];
    emb_a[0] = 1.0;
    let mut emb_b = vec![0.0f32; 512];
    emb_b[1] = 1.0;

    let mut track = Track::new(1, det_with_emb(0.0, 0.0, 0.9, 0, emb_a));
    track.add_detection(det_with_emb(5.0, 5.0, 0.1, 33, emb_b));
    let fused = fuse_track(&track);
    let v = fused.embedding.unwrap();
    // Component 0 should dominate over component 1.
    assert!(v[0] > v[1], "v[0]={} v[1]={}", v[0], v[1]);
}

#[test]
fn fuse_tracks_frame_count_correct() {
    let emb = unit_emb(0.5);
    let mut track = Track::new(1, det_with_emb(0.0, 0.0, 0.8, 0, emb.clone()));
    track.add_detection(det_with_emb(5.0, 5.0, 0.8, 33, emb.clone()));
    track.add_detection(det_with_emb(10.0, 10.0, 0.8, 66, emb));
    let fused = fuse_track(&track);
    assert_eq!(fused.frame_count, 3);
}
