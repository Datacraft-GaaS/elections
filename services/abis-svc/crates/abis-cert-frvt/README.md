# eel-abis-cert-frvt

NIST FRVT participation infrastructure for the eel ABIS stack.

## Modules

| Module | Track | Function |
|---|---|---|
| `frvt_1v1` | 1:1 Verification | TAR@FAR operating points + template pipeline |
| `frvt_1vn` | 1:N Identification | Rank-1 ID rate + FPIR@FNIR |
| `frvt_quality` | Quality | Mean quality + FMR correlation |
| `frvt_pad` | PAD | APCER/BPCER/ACER report card |

## Tests

Run with: `cargo test -p eel-abis-cert-frvt`

10+ tests covering TAR computation, rank-1 bounds, quality validation, PAD ACER.
