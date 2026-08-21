# eel-abis-fp-tps-warp

Thin-Plate Spline non-rigid distortion correction for fingerprint minutiae.

## Algorithm

- **TPS fitting**: Bookstein (IEEE TPAMI 1989). Minimum-bending interpolant solving the (N+3) x (N+3) linear system.
- **Registration**: rigid alignment (best single-pair) + nearest-neighbour correspondences + one IRLS refinement round.
- **Score improvement**: matcher score after TPS warp vs. without; the better of the two is reported.

## Usage

```rust
use eel_abis_fp_tps_warp::{Minutia, matcher::TpsWarpMatcher};

let matcher = TpsWarpMatcher::default();
let result = matcher.compare(&probe_minutiae, &gallery_minutiae)?;
println!("before={:.3} after={:.3} improved={}", result.score_before, result.score_after, result.improved);
```
