# hsm-signer — TKLA+ Proof Package

**Service**: HSM signer — PKCS#11 Ed25519+ML-DSA dual signature, CCB log, break-glass
**Audit ref**: M5 (auditId=g-tkla-hsm-signer, proofHash=sha256(specs), sig=HSM)
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
- `chaos/scenarios/insider-risk/08-hsm-key-extraction-attempt.yaml` — key extraction
- `chaos/scenarios/eel-hsm-unavailable.yaml` — PKCS#11 unavailability
- `chaos/scenarios/integration-failure/06-hsm-pkcs11-down.yaml` — signer down during declaration

## TLA+ Invariants
- `1. \A entry \in ccb_log : Cardinality(entry.approvers \cap ValidSigners) >= 2`
- `2. \A i \in 2..Len(ccb_log) : ccb_log[i].prev_hash = Hash(ccb_log[i-1])`
- `3. \A entry \in ccb_log : entry.final = TRUE => Cardinality(entry.approvers) >= 2`
- `4. \A sig \in signatures : verifyEd25519(sig) = TRUE /\ verifyMLDSA(sig) = TRUE`
- `5. break_glass_active = TRUE => emergency_signatures >= 1 /\ audit_logged = TRUE`
- `6. \A entry \in ccb_log : entry.audit_id /= NULL`

## Certora CVL Rules
- `spec multiSig2of3 { method f(e: CCBEntry) { assert count(e.approvers) >= 2; } }`
- `spec prevHashChain { method f(e: CCBEntry, prev: CCBEntry) { assert e.prev_hash == hash(prev); } }`
- `spec dualSignature { method f(s: Signature) { assert verifyEd25519(s) && verifyMLDSA(s); } }`
- `spec breakGlassAudit { method f() { assert break_glass_active == true => audit_logged == true; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
