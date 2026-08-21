# eel-abis-fp-multifinger

Multi-finger slap segmentation and weighted fusion.

## Algorithms

- **Slap segmentation**: Otsu threshold + vertical projection profile valley detection (ISO/IEC 29109-2:2018).
- **Finger fusion**: position-weighted average (thumb 1.2, index 1.1, middle 1.0, ring 0.9, little 0.8).
- **DPR gate**: rejects when fewer than K fingers match out of N available (NIST MINEX III protocol).

## Usage

```rust
use eel_abis_fp_multifinger::{GrayImage, matcher::MultiFingerMatcher, slap::SlapKind};

let m = MultiFingerMatcher::default();
let result = m.compare_slaps(&probe, &gallery, SlapKind::RightFour)?;
println!("score={}, matched={}, dpr_passed={}", result.overall_score, result.count_matched, result.dpr.passed);
```
