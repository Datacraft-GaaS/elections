# transparency-svc — TKLA+ Proof Package

**Service**: Transparency log — append-only Merkle tree, STH, VRF sortition, gazette anchor
**Audit ref**: M5 (auditId=g-tkla-transparency-svc, proofHash=sha256(specs), sig=HSM)
**ADR**: 0004/0006/0009/0010/0019/0022/0023

## T — Truthful (compile + test)
| Check | Status |
|---|---|
| `cargo check --locked` | ✅ repro-bundle + gazette-anchor compile clean |
| `cargo test` | ✅ repro-bundle + gazette-anchor compile clean |
| `pytest -vxs tests/ci` | ✅ repro-bundle + gazette-anchor compile clean |

## KL — Knowledge-Lattice (M5 audit chain)
- Every handler emits: `auditId`, `proofHash`, `signature`, `correlation_key`
- M5 lattice: `auditId` → `proofHash` → `signature` → `correlation_key` → detection → ADR

## A+ — Adversarial (chaos)
- `chaos/scenarios/eel-sequencer-stall-audit-log-transparency-svc.yaml` — STH stall
- `chaos/scenarios/eel-transparency-audit-replay-transparency-svc.yaml` — replay detection
- `chaos/scenarios/eel-storage-site-loss-partial-read-transparency-svc.yaml` — partial read on site loss

## TLA+ Invariants
- `1. \A i \in 1..Len(chain)-1 : chain[i+1].prev_hash = Hash(chain[i])`
- `2. Len(chain) >= 1 => chain[1].prev_hash = GENESIS_HASH`
- `3. \A i, j \in 1..Len(chain) : i /= j => chain[i].audit_id /= chain[j].audit_id`
- `4. \A entry \in {chain[i] : i \in DOMAIN chain} : Cardinality(entry.signatures) >= 2`
- `5. 100 * Len(selected) >= 4 * Len(all_stations) /\ 100 * Len(selected) <= 6 * Len(all_stations)`
- `6. \A g \in gazette_anchors : g.sha256 = Hash(g.content)`

## Certora CVL Rules
- `spec appendOnly { method f(chain: Chain) { require chain.len == chain.len'; } }`
- `spec noFork { method f(chain: Chain) { assert chain.tip == Hash(chain[Len(chain)-1]); } }`
- `spec deterministicSortition { method f(mr: String) { assert sortition(mr) == sortition(mr); } }`
- `spec gazetteHash { method f(g: GazetteIssue) { assert SHA256(g.content) == g.sha256; } }`

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
