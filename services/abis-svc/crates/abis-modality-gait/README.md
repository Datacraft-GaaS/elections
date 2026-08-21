# eel-abis-modality-gait

Gait (walking pattern) biometric modality crate for the EEL ABIS stack.

## Standard

Aligned with ISO/IEC 19794-6 (iris, adapted) joint-trajectory conventions. Uses 17 joints (COCO keypoint convention) with pitch/yaw/roll angles per frame, cycle-normalised.

## Parser

Binary format "GAIT": cadence f32 LE, walking_speed f32 LE, cycle count u16 LE, then per cycle (frame count u16 LE, duration_ms u32 LE, per frame (frame_idx u32 LE, 17 * 3 f32 LE joint angles)).

## Matcher Algorithm

DTW on 3D joint-angle sequences with cadence normalisation:

- Frame distance: mean squared error across all 51 joint-angle values (17 joints * 3 axes).
- DTW cost normalised by path length.
- Cadence factor: exp(-0.01 * |cadence_probe - cadence_gallery|).
- Score = cadence_factor * exp(-dtw_cost * 2.0).
- Default threshold: 0.40.

Best-cycle matching: the minimum-cost cycle pair is selected across all probe-gallery cycle combinations.

## Quality Scorer

Returns `GaitQuality`:

- `cycle_count_score`: 100 if 3-8 cycles, scaled down outside that range.
- `speed_score`: 100 if 1.0-2.0 m/s, penalised outside.
- `overall`: equally weighted average.

## Example Usage

```rust
use eel_abis_modality_gait::{parser, matcher};

let template = parser::parse_gait(&bytes)?;
let score = matcher::match_gait(&probe, &gallery);
println!("DTW cost: {:.4}, cadence diff: {:.1}", score.dtw_cost, score.cadence_diff);
```
