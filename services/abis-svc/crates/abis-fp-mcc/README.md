# eel-abis-fp-mcc

Minutia Cylinder-Code (MCC) fingerprint matching.

## Reference

R. Cappelli, M. Ferrara, D. Maltoni, "Minutia cylinder-code: a new representation and matching
technique for fingerprint recognition," IEEE Trans. Pattern Anal. Mach. Intell., vol. 32, no. 12,
pp. 2128-2141, Dec. 2010. DOI: 10.1109/TPAMI.2010.52

## Parameters (Table I defaults)

| Parameter | Default | Meaning |
|-----------|---------|---------|
| R (radius) | 70 px | Cylinder radius at 500 ppi |
| Ns | 16 | Spatial grid dimension |
| Nd | 6 | Directional layers |
| sigma_s | R/3 | Spatial Gaussian std-dev |
| sigma_d | 2*PI/Nd | Directional Gaussian std-dev |
| mu_psi | 0.001 | Cell activation threshold |
| tau_psi | 0.4 | Cylinder validity fraction |
| min_matched | 4 | Min selected pairs for non-zero score |

## Integration with abis-matcher

```rust
use eel_abis_fp_mcc::MccMatcher;

let matcher = MccMatcher::default();
let score = matcher.compare(&probe_template, &gallery_template);
// score in [0.0, 1.0]; typical match threshold ~0.60
```
