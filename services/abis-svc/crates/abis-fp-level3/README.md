# eel-abis-fp-level3

Level-3 fingerprint feature extraction and matching.

## Reference

A. K. Jain, Y. Chen, M. Demirkus, "Pores and Ridges: High-Resolution Fingerprint Matching
Using Level 3 Features," IEEE Trans. Pattern Anal. Mach. Intell., vol. 29, no. 1,
pp. 15-27, Jan. 2007. DOI: 10.1109/TPAMI.2006.224

## Features

- **Pore extraction**: local intensity minima on ridge crests, non-maximum suppressed
  with 8 px minimum spacing.
- **Ridge edge contours**: left/right edge points per ridge with curvature and width.
- **Incipient ridges**: nascent short ridges between mature ridges.
- **Quality assessment**: pore visibility + edge sharpness; minimum 500 ppi required.

## Resolution requirement

Level-3 features are only reliable at >= 500 ppi. The matcher returns
`Level3QualityError::InsufficientResolution` below this threshold.

## Usage

```rust
use eel_abis_fp_level3::{Level3Matcher, matcher::Level3Features, quality::Level3Quality};

let quality = Level3Quality::new(1000, 0.9, 0.85);
let features = Level3Features { pores, edge_points, quality };
let matcher = Level3Matcher::new();
let score = matcher.compare(&probe_features, &gallery_features, &probe_minutiae, &gallery_minutiae)?;
```
