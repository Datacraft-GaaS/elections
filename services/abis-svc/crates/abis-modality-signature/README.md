# eel-abis-modality-signature

Dynamic handwritten signature modality crate for the EEL ABIS stack.

## Standard

ISO/IEC 19794-7:2021 (Signature/Sign Time Series Data). Captures pressure, velocity, and acceleration time series from pen-tablet or on-screen capture devices.

## Parser

Binary format "SIG\x01" with f32 LE arrays (pressure, velocity, acceleration) and a pen-up bitmap. Parses into `SignatureTemplate` with per-sample structs including pen_up flag. Stroke count and pen_up_count are derived during parse.

## Matcher Algorithm

Dynamic Time Warping (DTW) over the tri-modal (pressure, velocity, acceleration) time series:

- Frame distance: Euclidean distance in 3D feature space.
- DTW normalised by path length.
- Score = exp(-dtw_cost * 2.0), mapping [0, inf) to (0, 1].
- Default threshold: 0.55.

Pen-up alignment: strokes are segmented at pen-up events and matched stroke-by-stroke before combining into a final DTW score.

## Quality Scorer

Returns `SignatureQuality`:

- `sample_rate_score`: 100 * (hz / 100).min(1.0).
- `duration_score`: 100 if 1-5 seconds, penalised outside.
- `stroke_score`: 100 if 3-8 strokes, penalised outside.
- `overall`: equally weighted average.

## Example Usage

```rust
use eel_abis_modality_signature::{parser, matcher};

let template = parser::parse_signature(&bytes)?;
let score = matcher::match_signature(&probe, &gallery);
```
