# eel-abis-modality-vein

Finger and palm vein biometric modality crate for the EEL ABIS stack.

## Standard

Aligned with ISO/IEC 19794-9:2011 (Vascular Biometric Data). Supports finger_vein, palm_vein, and wrist_vein subtypes captured in NIR (near-infrared) illumination.

## Parser

Binary NIR bitmap format with magic "VNIR", width/height/wavelength as u16 LE, VeinType and BodySide bytes, followed by raw grayscale pixel data.

## Matcher Algorithm

Normalised Cross-Correlation (NCC) over a rotation and translation search grid:

- Rotation search: [-15, -10, -5, 0, 5, 10, 15] degrees.
- Translation search: 5x5 grid at +/- 10 px steps.
- NCC = sum((A - meanA)(B - meanB)) / (n * stdA * stdB).
- Final score = max NCC across all grid positions.
- Default threshold: 0.60.

Score struct includes best_rotation_deg, best_tx, best_ty for forensic audit.

## Quality Scorer

Returns `VeinQuality`:

- `contrast`: pixel standard deviation / 128.
- `branch_density`: branch points per 100 pixels (a pixel is a branch point when it exceeds threshold 128 and has 3 or more qualifying 8-neighbors).
- `overall`: 50 * contrast + 50 * min(branch_density, 1.0).

## Example Usage

```rust
use eel_abis_modality_vein::{parser, matcher, quality};

let template = parser::parse_nir_bitmap(&bytes)?;
let score = matcher::match_vein(&probe, &gallery);
let q = quality::score_quality(&template);
```
