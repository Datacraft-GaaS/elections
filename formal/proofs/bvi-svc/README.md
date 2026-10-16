# bvi-svc — TKLA+ Proof Package

**Service**: Biometric verification identity — real-time capture, FAP60, anti-iris, anomaly detection
**Audit ref**: M5 (auditId=g-tkla-bvi-svc, proofHash=sha256(specs), sig=HSM)
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
- `chaos/scenarios/eel-pod-kill-bvi-svc.yaml` — pod kill
- `chaos/scenarios/eel-bvi-identification-storm-bvi-svc.yaml` — identification storm
- `chaos/scenarios/integration-failure/17-nist-beacon-timeout.yaml` — beacon timeout

## TLA+ Invariants
- `1. \A r \in verifications : r.score >= 0.0 /\ r.score <= 1.0`
- `2. \A r \in verifications : r.liveness_score >= 0.0 /\ r.liveness_score <= 1.0`
- `3. \A r \in verifications : r.fap60_result /= NULL => r.fap60_genuine = TRUE`
- `4. \A anomaly \in anomalies : anomaly.detected_at <= NOW`
- `5. \A anomaly \in anomalies : anomaly.confidence >= 0.0`

## Certora CVL Rules
- `spec scoreBounded { method f(r: VerificationResult) { assert 0.0 <= r.score && r.score <= 1.0; } }`
- `spec livenessPass { method f(r: VerificationResult) { assert r.liveness_score >= 0.5; } }`
- `spec anomalyConfidence { method f(a: Anomaly) { assert a.confidence >= 0.0 && a.confidence <= 1.0; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
