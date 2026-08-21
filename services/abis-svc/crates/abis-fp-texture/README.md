# eel-abis-fp-texture

Texture-based fingerprint matching: FingerCode, LBP, MB-LBP.

Used as a fallback when minutiae extraction quality is low (NFIQ2 score indicates poor quality).

## Algorithms

- **FingerCode** (Jain et al., IEEE TIP 2000): Gabor filterbank applied to 80 circular sectors around the reference point. Matching via Euclidean distance on 640-dimensional feature vector.
- **LBP**: Local Binary Patterns histogram (256 bins). Chi-square distance matching.
- **MB-LBP**: Multi-Block LBP at block sizes 3/5/9 for scale invariance. 768-dimensional multi-scale histogram.

## Usage

```rust
use eel_abis_fp_texture::{GrayImage, matcher::TextureMatcher};

let probe = GrayImage::new(width, height, pixels)?;
let gallery = GrayImage::new(width, height, pixels)?;
let matcher = TextureMatcher::default();
let result = matcher.compare(&probe, &gallery)?;
println!("fused score: {}", result.fused);
```
