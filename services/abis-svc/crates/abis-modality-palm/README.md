# eel-abis-modality-palm

Palm print biometric modality crate for the EEL ABIS stack.

## ISO Standard

ISO/IEC 19794-2:2011 Finger Minutiae Data (palm extension). The parser reads and writes the standard 6-byte minutia record layout (type+X packed as u16, Y as u16, angle as u8, quality as u8) with a 12-byte ISO header.

## Matcher Algorithm

Bozorth3-style pair matching adapted for palm prints:

- Rotation tolerance: +/- 30 degrees.
- Translation tolerance: +/- 80 pixels.
- Score: matched_pairs / min(probe_count, gallery_count), clamped to [0.0, 1.0].
- Crease bonus: +0.05 if crease pattern overlap exceeds 50% (XOR popcount heuristic).
- Default threshold: 0.40.

## Quality Scorer

Adapted NFIQ2 with five palm-zone weighting:

| Zone | Weight |
|------|--------|
| Writer | 0.30 |
| Upper | 0.20 |
| Lower | 0.20 |
| Thenar | 0.15 |
| Hypothenar | 0.15 |

Score = weighted average of (minutiae_count / 40).min(1.0) per zone, scaled to 0-100.

The `segment_zones` function partitions a template's minutiae by x/y position into the five named zones.

## Example Usage

```rust
use eel_abis_modality_palm::{parser, matcher, quality, domain::PalmTemplate};

let minutiae = parser::parse_iso19794_2(&bytes)?;
let probe = PalmTemplate { minutiae, .. };
let score = matcher::match_palm(&probe, &gallery);
if score.threshold_pass {
    println!("Palm match: {:.2}", score.raw);
}
let q = quality::score_quality(&probe);
println!("Quality: {:.1}/100", q);
```
