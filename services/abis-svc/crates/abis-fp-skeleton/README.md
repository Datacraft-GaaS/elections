# eel-abis-fp-skeleton

Fingerprint ridge topology graph: Zhang-Suen thinning + graph extraction + subgraph matching.

## Algorithms

- **Thinning**: Zhang-Suen parallel algorithm (Commun. ACM 1985).
- **Graph extraction**: nodes = endpoints + bifurcations; edges = ridge segments with length, curvature, start/end orientation.
- **Matching**: VF2 subgraph isomorphism for small graphs (<=20 nodes), greedy for large. Cordella et al. IEEE TPAMI 2004.

## Usage

```rust
use eel_abis_fp_skeleton::{skeleton::BinaryImage, matcher::SkeletonMatcher};

let probe_bin = BinaryImage::from_gray(w, h, &gray_pixels, 128)?;
let gallery_bin = BinaryImage::from_gray(w, h, &gray_pixels, 128)?;
let m = SkeletonMatcher::default();
let score = m.compare(&probe_bin, &gallery_bin)?;
println!("fused: {}", score.fused);
```
