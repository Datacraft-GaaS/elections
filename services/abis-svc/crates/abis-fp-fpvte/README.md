# eel-abis-fp-fpvte

NIST FpVTE + PFT-III validation harness for the eel ABIS fingerprint pipeline.

## References

- P. Grother, W. Fiumara, R. Micheals, "NIST Fingerprint Vendor Technology
  Evaluation 2003," NISTIR 7123, NIST, 2004.
- P. Grother, M. Indovina, "Proprietary Fingerprint Template Evaluation III,"
  NIST IR 8307, NIST, 2020.

## Tracks

| Track | Method |
|-------|--------|
| 1:1 Verification | FMR/FNMR ROC at 100 threshold steps |
| 1:N Identification | Rank-1 to Rank-10 accuracy |
| Fairness | Per-demographic FMR/FNMR slice |
| Report | JSON + CSV + plain-text summary |

## Supported corpus layouts

- Synthetic in-memory (for testing)
- Filesystem layout: `{root}/{subject_id}/{acquisition}.bin`
- Future: NIST SD-4, SD-14, SD-27, SD-29, SD-302, SD-361 native formats

## Matchers

Any type implementing `FpMatcher` can be evaluated: Bozorth3, MCC,
SourceAFIS-bridge, DL-ONNX, consensus.
