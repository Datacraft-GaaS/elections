# eel-abis-fp-consensus

Multi-algorithm consensus matcher: Bozorth3 + MCC + FingerCode + orientation field + skeleton graph.

## Design

Five sub-matchers run in sequence, each producing a score and binary vote:

| # | Matcher | Weight | Crate |
|---|---------|--------|-------|
| 1 | Bozorth3 | 1.0 | abis-matcher |
| 2 | MCC | 1.2 | eel-abis-fp-mcc |
| 3 | FingerCode texture | 0.8 | eel-abis-fp-texture |
| 4 | Orientation field | 0.7 | eel-abis-fp-orientation |
| 5 | Skeleton graph | 0.9 | eel-abis-fp-skeleton |

Scores are calibrated via Platt scaling, then fused by weighted average. A `MinAgreementPolicy` (default K=3 of N=5) must be satisfied for a match decision regardless of the fused score.

## Usage

```rust
use eel_abis_fp_consensus::consensus::{ConsensusMatcher, ConsensusInput};

let m = ConsensusMatcher::default();
let input = ConsensusInput {
    minutiae_probe: Some(&probe_tmpl),
    minutiae_gallery: Some(&gallery_tmpl),
    image_probe: Some(&gray_bytes),
    image_gallery: Some(&gray_bytes),
    image_width: 300,
    image_height: 300,
    binary_probe: None,
    binary_gallery: None,
};
let score = m.compare(&input)?;
println!("{}", score.explanation);
```
