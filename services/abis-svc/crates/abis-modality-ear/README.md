# eel-abis-modality-ear

Ear geometry biometric modality crate for the EEL ABIS stack.

## Standard

Aligned with ISO/IEC TR 29794-1 (quality) principles applied to ear contour shape. Uses 32 (x, y) landmark points around the ear contour.

## Parser

Binary format "EARL" (268 bytes): visibility_mask u32 LE, rotation_angle f32 LE, then 32 pairs of f32 LE coordinates. Invisible landmarks (mask bit = 0) still occupy their fixed slot (coordinates stored as 0,0).

## Matcher Algorithm

Procrustes-aligned Euclidean distance, using only mutually visible landmarks:

1. Centroid-centre both point sets.
2. Normalise by RMS distance from centroid.
3. Find optimal 2D rotation: theta = atan2(cross, dot) from paired coordinates.
4. Apply rotation and compute mean squared residual distance.
5. Score = exp(-procrustes_dist * 5.0).
6. Default threshold: 0.50.

## Quality Scorer

Returns `EarQuality`:

- `visibility_score`: (visible_count / 32) * 100.
- `rotation_score`: 100 * (1 - |angle_deg| / 90).max(0.0).
- `overall`: 0.7 * visibility + 0.3 * rotation.

## Example Usage

```rust
use eel_abis_modality_ear::{parser, matcher};

let template = parser::parse_ear(&bytes)?;
let score = matcher::match_ear(&probe, &gallery);
println!("Procrustes dist: {:.4}", score.procrustes_dist);
```
