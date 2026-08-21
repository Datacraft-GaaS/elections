# eel-abis-fp-orientation

Fingerprint orientation field analysis: singular points, pattern classification, field correlation matching.

## Algorithms

- **Orientation field**: gradient-based estimation (Rao 1990), block size configurable.
- **Singular points**: Poincare index (Kawagoe & Tojo 1984) detecting cores (+1) and deltas (-1).
- **Pattern classification**: Henry system (Arch, TentedArch, LeftLoop, RightLoop, Whorl, TwinLoop).
- **Matcher**: three-level (pattern class / singular points / field correlation).

## Usage

```rust
use eel_abis_fp_orientation::{GrayImage, matcher::OrientationMatcher};

let matcher = OrientationMatcher::default();
let score = matcher.compare(&probe_image, &gallery_image)?;
println!("fused: {}, passed_coarse: {}", score.fused, score.passed_coarse);
```
