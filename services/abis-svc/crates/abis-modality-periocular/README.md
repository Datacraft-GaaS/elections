# eel-abis-modality-periocular

Periocular (eye region without iris) biometric modality crate for the EEL ABIS stack.

## Standard

Aligned with ISO/IEC 19794-5 (Face Image Data) periocular region conventions. Fixed 128x128 grayscale patch with Local Binary Pattern (LBP) histogram descriptor.

## Parser

Binary format "PERI" with 16384 bytes of grayscale patch followed by 256 f32 LE LBP histogram bins, sharpness (f32 LE), and occlusion ratio (f32 LE).

`compute_lbp_histogram(patch, width, height)` is also exposed: iterates interior pixels, compares each with its 8 neighbors (setting bits 0-7 for neighbor >= center), increments histogram[code], then normalises to sum to 1.0.

## Matcher Algorithm

Chi-square distance on LBP histograms:

- chi_sq = sum((h1[i] - h2[i])^2 / (h1[i] + h2[i] + 1e-10))
- similarity = exp(-chi_sq / 2.0), mapping distance to [0, 1].
- Default threshold: 0.45.

## Quality Scorer

Returns `PeriocularQuality`:

- `sharpness_score`: Laplacian variance (kernel [0,1,0; 1,-4,1; 0,1,0]) clamped and normalised, 0-100.
- `occlusion_score`: 100 * (1 - occlusion_ratio).
- `overall`: 0.6 * sharpness + 0.4 * occlusion.

## Example Usage

```rust
use eel_abis_modality_periocular::{parser, matcher, quality};

let template = parser::parse_periocular(&bytes)?;
let lbp = parser::compute_lbp_histogram(&template.patch, 128, 128);
let score = matcher::match_periocular(&probe, &gallery);
```
