# ADR 0017: Transparency log

Status: accepted
Date: 2026-08-21

## Context

Public verifiability is an active defence — it makes disinformation attacks quantifiably falsifiable and gives observers, media, and courts an independent basis to confirm no rewrite happened. The spec (`sw/ARCHITECTURE.md:174`) mandates a CT-style Merkle transparency log. M14 reproducibility bundle depends on it. The log is also the first real service we plan to build — foundational, self-contained, exercises the full Rust + HSM + event-source + OpenAPI stack.

## Decision

**Single monolithic append-only Merkle log per election, with domain-tagged leaves.** Rust service under `services/transparency-log/`. Modelled on Trillian and Sigstore Rekor.

- **Storage**: PostgreSQL (in T1-consensus) for leaves + hash chain (append-only); MinIO for archived tree snapshots and full history export.
- **Leaf format**: `hash( canonical_json(event) || producer_spiffe_id || timestamp || sequence )`.
- **Domain tags on leaves**: `voter-events | results-events | custody-events | credential-events | meta-events`. External contract is one canonical log; internal storage may partition by domain for scale.
- **Signed Tree Head cadence**: **1 second hot / 60 second default / 5 minute batch**. Batch STHs consolidate multiple hot STHs into a single canonical snapshot for external witnesses.
- **STH signature**: HSM-signed (see ADR-0005). One HSM signature per STH covers up to millions of events. HSM throughput is not a constraint at this cadence.
- **Leaf signature**: producer-signed with SPIFFE Ed25519 key (see ADR-0015). Log verifies signature before hashing.
- **Split-view prevention**: STH cross-posted to independent witnesses — IAB service (day 1), external observers (day 2), Gazette daily anchor, optional blockchain anchor (deferred).
- **Third-party witness cosigning**: witnesses cosign STHs they observe; a fork produces divergent witness signatures, visible.
- **Consistency proofs** between any two STHs (verifier proves tree only grew).
- **Inclusion proofs** for any leaf (verifier proves event is in tree).
- **APIs**: gRPC (tonic) for internal producers/consumers; REST + OpenAPI (utoipa) for public verifiers, media, observers, M14 reproducibility bundle.
- **Public endpoints**:
  - `GET /api/v1/log/sth/latest` — latest STH per cadence tier.
  - `GET /api/v1/log/sth/{tree_size}` — historical STH.
  - `GET /api/v1/log/inclusion/{leaf_hash}` — inclusion proof.
  - `GET /api/v1/log/consistency/{from_size}/{to_size}` — consistency proof.
  - `GET /api/v1/log/entries?start=&count=` — bounded, rate-limited.
- **Availability**: log downtime does NOT block writes. Producer services buffer in outbox + Kafka; log ingest consumer catches up when the log recovers.
- **What goes on the log**:
  - Every T1 event from every event-sourced service.
  - Every credential issuance (kit, party agent, observer, staff).
  - Every STH from every domain (chain-of-chains).
  - Every schema migration on T1 clusters.
  - Every HSM key ceremony operation.
  - Every result declaration and every amendment.
- **What does NOT go on the log**:
  - PII (only hashes, never raw).
  - Ephemeral operational data.
  - Failed authz attempts (goes to SIEM instead).

## Consequences

Positive: single canonical log is simple to explain to auditors, simple to verify ("here is THE root"), single external contract; witness cosigning + Gazette anchoring makes split-view attacks visible; the M14 reproducibility bundle can rebuild the tree from raw events and compare to the published record; log availability is decoupled from write path.

Negative: log is a load-bearing service — its integrity is the election's integrity. Requires the strictest ops posture in the fleet: dedicated on-call rotation, monthly restore drill, quarterly witness cross-check drill. Sequencer performance is the internal scaling axis — internal storage sharding when needed, not external.

## Alternatives rejected

- Per-domain external logs (5-7 separate contracts): more scalable but harder to explain and verify externally; internal sharding provides scale without splitting the contract.
- HSM signature per leaf: architecturally impossible above ~15k events/sec regardless of budget.
- Blockchain-only publishing: reduces sovereign control; deferred to complement rather than replace the canonical log.
- Trillian direct fork: capable but Go-only; a Rust implementation aligns with the platform language default (ADR-0002).

## Related

ADR-0002 (Rust), ADR-0005 (HSM signs STHs), ADR-0012 (log leaves storage in T1-consensus), ADR-0013 (event-sourced modules feed the log), ADR-0015 (leaves are signed CloudEvents from the bus).
