# eel-governance-recall

M48 Recall petitions: Article 104 workflow, signatory de-duplication, 2M+ signature threshold verification, special election trigger.

## What it does

Manages recall petition life cycle: acceptance, signature collection and de-duplication, threshold checks, handoff to special-elections. Verifies signatory identity against voter register, flags duplicates and fakes, aggregates signatures across batches, and triggers SpecialElection creation when threshold (2M+ or 50%+ of registered voters, per Art. 104) is crossed.

Fits in governance-svc as the recall engine. Runs on port 8093; integrates with bvr-svc for voter identity checks (async gRPC calls with circuit breaker).

## Public API

`eel_governance_recall::petition::RecallPetition` aggregate: `fn add_signature_batch(&mut self, batch: SignatureBatch) -> Result<()>`, `fn is_threshold_crossed(&self) -> bool`, `fn trigger_special_election(&mut self) -> Result<SpecialElection>`.

Endpoints:
- `GET /v1/governance/recall/petitions` - list active
- `POST /v1/governance/recall/petitions` - initiate
- `GET /v1/governance/recall/petitions/{id}` - fetch status
- `POST /v1/governance/recall/petitions/{id}/signatures` - batch upload
- `POST /v1/governance/recall/petitions/{id}/declare-threshold-crossed` - admin trigger

Signature batches accepted as JSON arrays or CSV upload (parsed server-side). Dedup via voter ID + biometric template hash.

## Feature flags

`#[cfg(feature = "bvr-integration")]` for live voter register checks; tests use local fixture voters.

## Test commands

```bash
cargo test -p eel-governance-recall --all-features
cargo test -p eel-governance-recall --lib signature_dedup
cargo test -p eel-governance-recall --test integration_recall -- --include-ignored
```

Mock bvr-svc responses; verify dedup correctness on synthetic voter batches.

## Bench commands

```bash
cargo bench -p eel-governance-recall dedup_1M_signatures
```

De-duplication via bloom filter + database lookup should process 1M signatures in < 30s.

## Dependencies of note

- `uuid6`: petition and signature IDs
- `tokio`: bvr gRPC client, retry middleware
- `csv`: signature batch parsing
- `bloom`: probabilistic dedup pre-filter

## Related docs

- `docs/api-reference.md#recall` (petition endpoints)
- `migration_003_recall_petitions.sql` (schema: petitions, signatures, batches)
- Constitution Art. 104 text (invoked in comments)
