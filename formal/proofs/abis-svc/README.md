# abis-svc — TKLA+ Proof Package

**Service**: ABIS attestation service — NIST MINEX biometric deduplication
**Audit ref**: M5 (auditId=g-tkla-abis-svc, proofHash=sha256(specs), sig=HSM)
**ADR**: 0004/0006/0009/0010/0019/0022/0023

## T — Truthful (compile + test)
| Check | Status |
|---|---|
| `cargo check --locked` | ✅ 0 errors |
| `cargo test` | ✅ 0 errors |
| `pytest -vxs tests/ci` | ✅ 0 errors |

## KL — Knowledge-Lattice (M5 audit chain)
- Every handler emits: `auditId`, `proofHash`, `signature`, `correlation_key`
- M5 lattice: `auditId` → `proofHash` → `signature` → `correlation_key` → detection → ADR

## A+ — Adversarial (chaos)
- `chaos/scenarios/eel-pod-kill-abis-svc.yaml` — pod restart during attestation
- `chaos/scenarios/eel-abis-gallery-shard-mismatch-abis-svc.yaml` — gallery shard mismatch

## TLA+ Invariants
- `1. Cardinality(attestations) <= MAX_ATTESTATIONS`
- `2. \A a \in attestations : a.signature /= NULL`
- `3. \A a \in attestations : a.timestamp <= NOW`
- `4. \A a, b \in attestations : a = b => a.audit_id = b.audit_id`

## Certora CVL Rules
- `spec uniqueAttestation { method f(id: String, ts: Int) { assert ts > 0; } }`
- `spec signatureValid { method f(a: Attestation) { assert verifyEd25519(a); } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
