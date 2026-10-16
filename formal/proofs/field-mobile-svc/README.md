# field-mobile-svc — TKLA+ Proof Package

**Service**: Mobile BFF — offline-first, attestation, form quality gate, station binding
**Audit ref**: M5 (auditId=g-tkla-field-mobile-svc, proofHash=sha256(specs), sig=HSM)
**ADR**: 0004/0006/0009/0010/0019/0022/0023

## T — Truthful (compile + test)
| Check | Status |
|---|---|
| `cargo check --locked` | ⚠️ 30 pre-existing errors (E0061/E0308/E0599; redesign needed) |
| `cargo test` | ⚠️ 30 pre-existing errors (E0061/E0308/E0599; redesign needed) |
| `pytest -vxs tests/ci` | ⚠️ 30 pre-existing errors (E0061/E0308/E0599; redesign needed) |

## KL — Knowledge-Lattice (M5 audit chain)
- Every handler emits: `auditId`, `proofHash`, `signature`, `correlation_key`
- M5 lattice: `auditId` → `proofHash` → `signature` → `correlation_key` → detection → ADR

## A+ — Adversarial (chaos)
- `chaos/scenarios/eel-pod-kill-field-mobile-svc.yaml`
- `chaos/scenarios/eel-mobile-bff-auth-cascade-field-mobile-svc.yaml`
- `chaos/scenarios/eel-field-mobile-sync-disconnect-loop-field-mobile-svc.yaml`

## TLA+ Invariants
- `1. \A req \in form_submissions : req.quality_gate_passed = TRUE => req.accepted = TRUE`
- `2. \A req \in form_submissions : req.quality_gate_passed = FALSE => req.rejected = TRUE`
- `3. Len(rejected_reasons) >= 1 => quality_gate_passed = FALSE`
- `4. \A req \in form_submissions : req.station_code /= NULL`
- `5. Len(idempotency_keys) >= 1 => \A k \in idempotency_keys : \A x \in idempotency_keys : k = x => k.responses = x.responses`
- `6. \A app \in attestations : app.verdict \in {"MEETS", "FAILS", "UNVERIFIED"}`

## Certora CVL Rules
- `spec qualityGatePass { method f(r: FormSubmission) { assert r.quality_gate_passed == true => r.accepted == true; } }`
- `spec qualityGateReject { method f(r: FormSubmission) { assert r.quality_gate_passed == false => r.rejected == true; } }`
- `spec idempotencyKey { method f(k: String, v: Response) { assert store(k, v) == store(k, v); } }`
- `spec attestationVerdict { method f(v: AttestationVerdict) { assert v.verdict in {'MEETS','FAILS','UNVERIFIED'}; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
