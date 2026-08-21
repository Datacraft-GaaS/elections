# archive

Long-term archive crate for M69. Manages the 15-year statutory retention of all election records, with KNADS integration, DataCite DOI issuance, BagIt packaging, post-quantum re-signing (SLH-DSA), and preservation attestations logged to the transparency log.

## What it does

M69 is the terminal sink for election evidence: final tally snapshots, kit event logs, biometric audit trails, result declarations, campaign finance records, and ballot serial chains. Records arrive from rts-svc, transparency-svc, bvr-svc, and inventory-svc. The crate packages them as BagIt bags, computes SPDX manifests, issues DataCite DOIs, ships to KNADS, and produces preservation attestations that are appended to the Merkle log.

SLH-DSA re-signing is applied on a configurable schedule (default: at 5-year intervals) to ensure archived signatures remain verifiable as classical algorithms age.

## Public API surface

- `mod ingest` - `IngestJob`, `IngestJobBuilder`, `ingest_batch()`
- `mod collection` - `ArchiveCollection`, `CollectionStatus`, `list_collections()`
- `mod preservation` - `PreservationAttestation`, `attest_collection()`, `resign_collection()`
- `mod destruction` - `DestructionCertificate`, `schedule_destruction()` (ODPC §40 right to erasure)
- `mod doi` - DataCite DOI registration and metadata update
- `mod bagit` - BagIt bag creation, validation, and transfer to KNADS

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `knads-live` | off | Enable live KNADS FTP transfer (requires MoU credentials) |
| `datacite-live` | off | Enable live DataCite DOI registration |
| `pq-resign` | on | Enable SLH-DSA re-signing pipeline |

## Test commands

```bash
cargo nextest run -p archive
cargo nextest run -p archive --test integration -- --include-tag archive
```

## Bench commands

```bash
cargo bench -p archive -- bagit_pack
cargo bench -p archive -- ingest_throughput
```

## Dependencies of note

- `eel-crypto-rs` (workspace): SLH-DSA re-signing, cosign verify
- `eel-events-rs` (workspace): event envelope and HLC
- `eel-pii-rs` (workspace): PIIRestricted column handling before export
- `eel-outbox-rs` (workspace): preservation attestation publishing
- `bagit` (external): BagIt bag format implementation
- `sha2`, `blake3` (external): manifest hashing

## Related docs

- `docs/plans/modules/M69-long-term-archive.md`
- `services/support-admin-svc/py/bagit/` - Python BagIt helper scripts
- `packages/event-schemas/support/archive.*.json`
- `docs/runbooks/support-admin-svc-archive-recovery.md`
