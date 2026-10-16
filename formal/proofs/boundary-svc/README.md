# boundary-svc — TKLA+ Proof Package

**Service**: boundary-svc — no invariants specified yet
**Audit ref**: M5 (auditId=g-tkla-boundary-svc, proofHash=sha256(specs), sig=HSM)
**ADR**: 0004/0006/0009/0010/0019/0022/0023

## T — Truthful (compile + test)
| Check | Status |
|---|---|
| `cargo check --locked` | — |
| `cargo test` | — |
| `pytest -vxs tests/ci` | — |

## KL — Knowledge-Lattice (M5 audit chain)
- Every handler emits: `auditId`, `proofHash`, `signature`, `correlation_key`
- M5 lattice: `auditId` → `proofHash` → `signature` → `correlation_key` → detection → ADR

## A+ — Adversarial (chaos)
- (none specified)

## TLA+ Invariants


## Certora CVL Rules


## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
