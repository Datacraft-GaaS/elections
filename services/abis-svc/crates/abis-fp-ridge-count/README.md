# eel-abis-fp-ridge-count

Bozorth3-style ridge counting between minutiae and K-plet descriptor matching.

## Reference

- C. Watson et al., "User's Guide to NIST Biometric Image Software (NBIS)," NISTIR 7392,
  NIST, 2007. (Bozorth3 ridge counting.)
- N. K. Ratha, R. M. Bolle, V. D. Pandit, V. Vaish, "Robust fingerprint authentication
  using local structural similarity," IEEE Workshop on AVSPA, 2000. (K-plet descriptors.)

## Modules

- **count**: `count_ridges(from, to, orientation_field)` traces the line between two
  minutiae and counts ridge crossings via perpendicular zero-crossings in the orientation
  field. Falls back to a geometric estimate (distance / 15 px) when no field is provided.

- **kplet**: For each minutia, sorts its K (default 5) nearest neighbours by distance and
  records (rho, phi, theta_diff, ridge_count) tuples.

- **matcher**: `KPletMatcher` compares templates via K-plet tuple set intersection.
  `RidgeCountMatcher` compares the stored ridge-count matrices directly.

## Usage

```rust
use eel_abis_fp_ridge_count::{count_ridges, OrientationField, matcher::KPletMatcher};

let field = OrientationField::new(width, height, orientation_data);
let matcher = KPletMatcher::new(field);
let score = matcher.compare(&probe, &gallery);
```
