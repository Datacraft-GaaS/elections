# inventory-svc — TKLA+ Proof Package

**Service**: Inventory tracking — RFID, ballot chain-of-custody, HMAC-SHA256, M-Pesa B2C
**Audit ref**: M5 (auditId=g-tkla-inventory-svc, proofHash=sha256(specs), sig=HSM)
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
- `chaos/scenarios/eel-pod-kill-inventory-svc.yaml`
- `chaos/scenarios/eel-asset-return-seal-failure-inventory-svc.yaml`
- `chaos/scenarios/integration-failure/09-mpesa-daraja-timeout.yaml`

## TLA+ Invariants
- `1. \A h \in handovers : hmac(h, HMACKey) = h.hmac`
- `2. \A i \in 1..Len(chain)-1 : chain[i+1].prev_hash = Hash(chain[i])`
- `3. Len(chain) >= 1 => chain[1].seq_no = 1`
- `4. \A i \in 1..Len(chain)-1 : chain[i+1].seq_no = chain[i].seq_no + 1`
- `5. \A payment \in payments : payment.response_code \in {"0", "1", "17", "INSF"}`
- `6. \A payment \in payments : payment.audit_id /= NULL`

## Certora CVL Rules
- `spec hmacIntegrity { method f(h: BallotHandover, key: Bytes) { assert hmac_sha256(h, key) == h.hmac; } }`
- `spec seqNoMonotonic { method f(chain: Chain) { assert chain[i+1].seq == chain[i].seq + 1; } }`
- `spec paymentAudit { method f(p: Payment) { assert p.audit_id /= NULL; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
