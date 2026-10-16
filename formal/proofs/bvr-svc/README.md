# bvr-svc — TKLA+ Proof Package

**Service**: Biometric voter registration — ABIS deduplication, nearform-nbis FFI, gallery management
**Audit ref**: M5 (auditId=g-tkla-bvr-svc, proofHash=sha256(specs), sig=HSM)
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
- `chaos/scenarios/eel-bvr-enrolment-overflow-bvr-svc.yaml` — enrolment overflow
- `chaos/scenarios/eel-credential-revocation-storm-bvr-svc.yaml` — credential revocation storm
- `chaos/scenarios/insider-risk/11-gallery-poisoning-attempt.yaml` — adversarial template injection

## TLA+ Invariants
- `1. \A v \in voters : v.biometric_template /= NULL`
- `2. \A v \in voters : v.audit_id /= NULL`
- `3. \A v \in voters : \A u \in voters : v /= u => v.national_id /= u.national_id`
- `4. Cardinality(dedup_candidates) >= 2 => duplicate_found = TRUE`
- `5. \A v \in verified : verify_nbis(v.template, v.fingerprint) = TRUE`

## Certora CVL Rules
- `spec templateExists { method f(v: Voter) { assert v.template /= NULL; } }`
- `spec dedupFair { method f(gallery: Set[Voter], newV: Voter) { assert bozorth(gallery, newV); } }`
- `spec auditEmitted { method f(v: Voter) { assert v.audit_id /= NULL; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
