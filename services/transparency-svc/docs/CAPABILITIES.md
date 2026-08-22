# transparency-svc capabilities

Append-only Merkle transparency log with HSM-backed signing, five-channel STH publication, and campaign finance disclosure. Every event written to the electoral platform is provably included in this log.

**Tier**: T1-election-critical (transparency log + HSM signer); T2-routine (campaign finance, reproducibility bundle).
**SLO**: 99.99% availability. STH publication latency p99 < 1s (1s cadence), < 60s (60s cadence).
**RTO**: 30 min. **RPO**: 5 min.

---

## Contents

1. Purpose and context
2. End-to-end pipelines
3. Domain aggregates and state stores
4. Algorithms and standards
5. Crypto and trust anchoring
6. Event surface
7. API surface
8. Integrations
9. Sharding, indexing, and scale
10. Degradation modes
11. Data lifecycle and retention
12. Observability
13. Testing pyramid
14. Follow-ups and known limitations

---

## 1. Purpose and context

### Modules owned

| Module | Name | RFP ref |
|--------|------|---------|
| P05 | Transparency log | WP-2.1 |
| P06 | HSM signer | WP-2.2 |
| M14 | Reproducibility bundle | WP-6.1 |
| M22 | Campaign finance transparency | WP-7.3 |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-18 months | Key ceremony: M-of-N HSM keyholder shards distributed to Commission, IAB, and court-appointed witnesses |
| T-12 months | Log open for pilot events; campaign finance disclosure pipeline goes live |
| T-3 months | STH five-channel publication tested end-to-end including Gazette print anchor and KBC radio |
| Election day (T+0) | Every Form 34A append lands in log within 1s; STH published every 1s; IAB cosigns hourly |
| T+7 days | Declaration appended; reproducibility bundle published; log sealed with final STH |
| T+90 days | Campaign finance final reports due; all disclosures verified and anchored |

---

## 2. End-to-end pipelines

### 2.1 Event ingest and Merkle append

```
producer service -> Kafka topic (eel-tlog-ingest) -> tlog-ingest crate
  -> canonicalize leaf (RFC 8785 JCS) -> compute leaf hash (SHA-256)
  -> tlog-sequencer: assign index, insert into leaves table
  -> tlog-domain: update tree_tiles (Merkle tile cache)
  -> compute new root hash -> sign STH (HSM Ed25519 + ML-DSA hybrid)
  -> insert into sth table -> tlog-witness: collect cosignatures
  -> five-channel publisher: website / GitHub / IPFS-KE / Gazette / KBC
```

### 2.2 Inclusion proof serving

```
client GET /v1/tlog/proof/{leaf_hash}
  -> tlog-api: resolve leaf index from leaves table
  -> tlog-domain: walk tile tree to root (O(log n) tile reads)
  -> return MerkleAuditProof (sibling hashes + tree_size + root_hash + STH signature)
```

### 2.3 Campaign finance disclosure ingest

```
filer POST /v1/campaign-finance/disclosures (multipart)
  -> finance-transparency crate: validate schema + IEBC forms compliance
  -> persist to disclosures table -> compute SHA-256 of payload
  -> append disclosure hash to transparency log (as domain=finance-events leaf)
  -> emit campaign.disclosure.ingested CloudEvent via outbox
```

### 2.4 Reproducibility bundle generation

```
election sealed (rts.declaration.signed received)
  -> repro-bundle crate: collect all build artifacts, test harnesses, migration checksums
  -> compute BLAKE3 tree over bundle contents
  -> sign bundle manifest (HSM Ed25519)
  -> upload to MinIO + IPFS-KE
  -> append bundle hash to transparency log
  -> publish bundle STH and cosign request to witnesses
```

---

## 3. Domain aggregates and state stores

All tables are in `services/transparency-svc/migrations/V1__initial_schema.sql`.

| Table | Purpose |
|-------|---------|
| `leaves` | Append-only log leaves. Columns: `id`, `index` (monotonic sequence), `domain`, `hash` (SHA-256 of canonical JSON leaf), `producer_spiffe_id`, `source_event_id`, `source_topic`, `event_time`, `appended_at`, `producer_signature`. Unique on `(source_topic, source_event_id)` for idempotency. |
| `tree_tiles` | Tile cache for Merkle tree. Columns: `level`, `tile_index`, `hash` (SHA-256 of concatenated child hashes), `computed_at`. Primary key `(level, tile_index)`. |
| `sth` | Signed Tree Heads. Columns: `id`, `tree_size`, `root_hash`, `domain`, `cadence` (1s, 60s, 5min), `signed_at`, `signature` (Ed25519 + ML-DSA hybrid), `key_role`, `key_fingerprint`. Indexed by `tree_size DESC`. |
| `sth_batch` | Parent-child STH relationships for consistency proof chains. |
| `witnesses` | Registered cosigning witnesses. Columns: `witness_id`, `witness_spiffe_id`, `public_key_hex`, `is_internal`. |
| `witness_cosigs` | Per-STH witness cosignatures. Columns: `sth_id`, `witness_id`, `signature`, `cosigned_at`, `is_internal`. Unique on `(sth_id, witness_id)`. |
| `ingest_offsets` | Kafka consumer group offsets per `(topic, partition)` for at-least-once-then-deduplicate ingest. |
| `domains` | Registered log domains (voter-events, results-events, custody-events, credential-events, meta-events). |
| `disclosures` (finance-transparency crate) | Campaign finance disclosure records. Planned: `disclosure_id`, `filer_id`, `amount`, `category`, `period`, `payload_hash`, `tlog_leaf_index`, `submitted_at`. |
| `repro_bundles` (repro-bundle crate) | Reproducibility bundle manifests. Planned: `bundle_id`, `election_id`, `primary_hash`, `bundle_signature`, `ipfs_cid`, `published_at`. |

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| Merkle leaf hash | `crates/tlog-domain` | SHA-256 over RFC 8785 canonical JSON | Laurie, Langley, Kasper, RFC 6962, IETF 2013 | RFC 6962-bis |
| Tile-based Merkle tree | `crates/tlog-domain` | Tile tree with 256-ary internal nodes (log2 fan-out) | Crosby, Wallach, "Efficient Data Structures for Tamper-Evident Logging", USENIX Security 2009 | RFC 6962-bis |
| Consistency proof | `crates/tlog-domain` | Subproof from stored tile cache | Merkle 1989; IETF RFC 6962 Sec 2.1 | RFC 6962-bis |
| Canonical JSON | `crates/tlog-domain` | RFC 8785 JCS (JSON Canonicalization Scheme) | Rundgren, Jordan, Erdtman, RFC 8785, IETF 2020 | RFC 8785 |
| Primary signing | `crates/hsm-signer` | Ed25519 (PKCS#11 CKM_EDDSA) | Bernstein, Duif, Lange, Schwabe, Yang, EUROCRYPT 2011 | ISO/IEC 19790, FIPS 186-5 |
| Hybrid signing (PQC) | `crates/hsm-signer` | ML-DSA (CRYSTALS-Dilithium) + ECDSA P-384 dual signature | Ducas, Kiltz, Lepoint, Lyubashevsky, Schwabe, Seiler, Stehle, EUROCRYPT 2018 | FIPS 204, NIST PQC |
| Timestamp authority | `crates/hsm-signer` | RFC 3161 TSA token embedding | Adams, Cain, Pinkas, Zuccherato, RFC 3161, IETF 2001 | RFC 3161 |
| HSM PKCS#11 | `crates/hsm-signer` | PKCS#11 v3.0 key operations (C_Sign, C_Verify, C_GenerateKeyPair) | OASIS PKCS#11 TC, v3.0 2020 | ISO/IEC 19790 |
| Cosign witness format | `crates/tlog-witness` | cosign bundle format + in-toto attestation | Dex et al., "cosign: Container Signing, Verification, and Storage", OpenSSF 2021 | in-toto ITE-6 |
| Campaign finance schema | `crates/finance-transparency` | JSON Schema + IEBC forms compliance rules | planned; no crate file verified on disk yet | Kenya Elections Act 2011 |
| Reproducibility bundle | `crates/repro-bundle` | BLAKE3 content tree + Ed25519 manifest signature | O'Connor, Ausmasson, Neves, Wilcox-O'Hamill, "BLAKE3: One Function, Fast Everywhere", 2020 | planned |

---

## 5. Crypto and trust anchoring

### HSM key hierarchy

| Key role | Algorithm | HSM slot | Purpose |
|----------|-----------|----------|---------|
| `sth` | Ed25519 (primary) + ML-DSA (PQC backup) | Slot 0 (online) | Sign Signed Tree Heads at 1s/60s cadence |
| `declaration-witness` | Ed25519 | Slot 1 (online) | Countersign declarations forwarded from rts-svc |
| `bundle` | Ed25519 | Slot 1 (online) | Sign reproducibility bundle manifests |
| `kek-ceremony` | ECDH P-384 | HSM ceremony (offline, M-of-N) | Per-election-cycle KEK: wraps all service DEKs |

### Key ceremony

M-of-N key ceremony (minimum 3-of-5): Commission HSM custodian, IAB representative, court-appointed witness A, court-appointed witness B, backup custodian. Each keyholder holds one PKCS#11 key shard. Ceremony protocol uses Shamir Secret Sharing (Shamir, Communications of the ACM, 1979) over P-384. Ceremony logs appended to meta-events log domain with HSM signature.

### Event signing

Every `leaves` row carries `producer_signature` (Ed25519 by the producing service's SPIRE SVID private key) and `producer_spiffe_id` verified against the SPIRE trust bundle before append.

### Transparency log anchoring

STH publication to five channels serves as external anchoring. The canonical order:
1. IEBC website (HTTPS, < 1s)
2. GitHub public repository (GitHub Actions push, < 60s)
3. IPFS-KE node (IPFS CID pinned, < 5 min)
4. Kenya Gazette (print hash bulletin, next business day)
5. KBC Radio (hash bulletin read on air, daily)

---

## 6. Event surface

### Emitted (via transactional outbox, Kafka)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.tlog.leaf.appended.v1` | eel-tlog-events | `leaf_index`, `domain`, `hash`, `tree_size`, `root_hash` |
| `eel.tlog.sth.published.v1` | eel-tlog-events | `sth_id`, `tree_size`, `root_hash`, `cadence`, `signed_at`, `signature_hex` |
| `eel.tlog.witness.cosigned.v1` | eel-tlog-events | `sth_id`, `witness_id`, `cosigned_at`, `is_internal` |
| `eel.tlog.sth.channel.website.v1` | eel-tlog-publication | `sth_id`, `channel`, `published_at`, `url` |
| `eel.tlog.sth.channel.github.v1` | eel-tlog-publication | `sth_id`, `channel`, `commit_sha`, `published_at` |
| `eel.tlog.sth.channel.ipfs.v1` | eel-tlog-publication | `sth_id`, `channel`, `ipfs_cid`, `published_at` |
| `eel.campaign.disclosure.ingested.v1` | eel-transparency-finance | `disclosure_id`, `filer_id`, `amount`, `category`, `tlog_leaf_index` |
| `eel.hsm.key.rotated.v1` | eel-transparency-hsm | `key_role`, `old_fingerprint`, `new_fingerprint`, `rotated_at` |
| `eel.repro.bundle.published.v1` | eel-tlog-events | `bundle_id`, `election_id`, `primary_hash`, `ipfs_cid`, `tlog_leaf_index` |

All envelopes follow CloudEvents 1.0 (CNCF CloudEvents Specification 1.0, 2020). Envelope: `specversion=1.0`, `type=<event type>`, `source=spiffe://eel.local/svc/transparency-svc`, `id=<UUID7>`, `time=<RFC 3339>`, `datacontenttype=application/json`.

### Consumed

| Event type | Topic | Action |
|------------|-------|--------|
| `eel.rts.form34.published.v1` | eel-rts-events | Trigger leaf append for result publication |
| `eel.rts.declaration.signed.v1` | eel-rts-events | Trigger declaration leaf append and reproducibility bundle start |
| `eel.bvr.register.frozen.v1` | eel-bvr-events | Record freeze window; gate result acceptance window |

---

## 7. API surface

### REST (tlog-api crate, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/tlog/sth` | Latest Signed Tree Head |
| GET | `/v1/tlog/sth/by-timestamp` | STH by Unix timestamp (1s or 60s cadence) |
| GET | `/v1/tlog/proof/{leaf_hash}` | Merkle inclusion proof for leaf hash |
| GET | `/v1/tlog/consistency/{tree_size1}/{tree_size2}` | Consistency proof between two tree snapshots |
| POST | `/v1/tlog/append` | Append leaf (internal; SPIRE SVID required, non-public) |
| GET | `/v1/campaign-finance/disclosures` | List campaign finance disclosures (paginated) |
| GET | `/v1/campaign-finance/disclosures/{id}` | Single disclosure record |
| POST | `/v1/campaign-finance/disclosures` | Ingest disclosure (authenticated; filer role) |
| GET | `/v1/repro-bundle/{election_id}` | Download reproducibility bundle manifest |
| GET | `/healthz` | Liveness |
| GET | `/readyz` | Readiness (checks DB, HSM, Kafka) |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### gRPC (proto/transparency.proto, port 8443)

| RPC | Input | Output |
|-----|-------|--------|
| `AppendLeaf` | `leaf_bytes: bytes, producer_sig: bytes` | `LeafIndex { index: uint64, root_hash: bytes }` |
| `GetSTH` | `timestamp_ns: int64, cadence: string` | `SignedTreeHead { tree_size, root_hash, signature, key_fingerprint }` |
| `GetInclusionProof` | `leaf_hash: bytes, tree_size: uint64` | `AuditProof { leaf_index, audit_path: bytes[], root_hash }` |
| `PublishSTH` | `sth: SignedTreeHead` | `PublicationResult { channels_published: string[] }` |
| `IngestCampaignDisclosure` | `form_data: bytes, filer_id: string` | `DisclosureId { id: string, tlog_leaf_index: uint64 }` |

### HSM signer (crates/hsm-signer, internal gRPC, port 9443)

Signer is co-deployed as a sidecar. Only tlog-sequencer holds the SPIRE SVID that authorizes the `Sign` RPC. Key ceremony operations use a separate break-glass CLI with HSM operator authentication.

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| rts-svc (form publication) | Kafka consumer in `crates/tlog-ingest` | Kafka topic `eel-rts-events` |
| bvr-svc (register freeze) | Kafka consumer in `crates/tlog-ingest` | Kafka topic `eel-bvr-events` |
| IPFS-KE node | HTTP client in `crates/tlog-witness` | IPFS HTTP API v1 |
| GitHub Actions (STH publish) | HTTP webhook in `crates/tlog-witness` | GitHub REST API v3 |
| KBC Radio bulletin system | Email/SFTP adapter (planned) | SMTP + SFTP |
| OpenBao (HSM key material) | `crates/hsm-signer` via PKCS#11 | PKCS#11 v3.0 |
| SPIRE (SVID validation) | `crates/tlog-api` trust bundle check | SPIFFE workload API |

---

## 9. Sharding, indexing, and scale

The transparency log is a single authoritative append-only sequence. Horizontal sharding would break the sequential index property. Scale is achieved through:

- **Tile caching**: `tree_tiles` table holds pre-computed Merkle tiles; proof computation is O(log n) tile reads with no full-tree traversal.
- **Read replicas**: STH reads and proof serving served from PG read replicas; writes go only to primary.
- **Kafka partitioning**: `eel-tlog-ingest` topic partitioned by `domain` (5 domains = 5 partitions). Sequencer holds a single-consumer group to preserve global ordering.
- **Scale targets**: 22M voter events over a 90-day enrolment window (2.8 events/s average, 50 events/s burst); 46,229 Form 34A submissions on election day (worst-case 6 hr window = 2.1/s average, 500/s burst at polls close); 60s STH latency budget at all loads.
- **Index**: `sth_tree_size_idx` on `sth(tree_size DESC)` for fast latest-STH lookup. `leaves` unique on `(source_topic, source_event_id)` for idempotency.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | STH at 1s cadence; five-channel pub active; cosigning live | docs/runbooks/transparency-svc.md |
| Degraded (HSM signer unavailable) | HSM PKCS#11 connection lost | Ingest continues; STH signing suspended; alert fires; fallback to 60s cadence once HSM restored | docs/runbooks/transparency-svc.md#hsm-recovery |
| Degraded (IPFS-KE or GitHub unavailable) | Channel publish fails | Other channels continue; failed channel retried with exponential backoff; alert after 3 failures | docs/runbooks/transparency-svc.md#channel-recovery |
| Read-only (DB primary unavailable) | PG primary failover in progress | STH reads served from replica; new leaf appends queued in Kafka; ingest resumes once primary restored | docs/runbooks/transparency-svc.md#db-failover |
| Emergency (complete outage) | All primary components down | Pre-cached STHs served from CDN; last signed STH considered authoritative until recovery; RTO 30 min | docs/runbooks/transparency-svc.md#emergency |

---

## 11. Data lifecycle and retention

| Table | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `leaves` | Class A: election-critical permanent | Permanent (never deleted; log is append-only) |
| `tree_tiles` | Class A | Permanent (required for proof computation) |
| `sth` | Class A | Permanent |
| `witness_cosigs` | Class A | Permanent |
| `witnesses` | Class B: operational | 7 years post-election cycle |
| `ingest_offsets` | Class C: ephemeral | 30 days |
| `disclosures` | Class B | 10 years (Kenya Elections Act) |
| `repro_bundles` | Class A | Permanent |
| `domains` | Class B | Permanent (configuration) |

No PII is stored directly in transparency-svc. Leaf hashes are SHA-256 over producer payloads; the payloads themselves live in originating services.

---

## 12. Observability

### Dashboards (Grafana, d/transparency)

- Log ingest rate (leaves/s) by domain.
- STH signing latency histogram (target p99 < 1s).
- Five-channel publication lag per channel.
- Witness cosigning coverage (IAB vs external).
- HSM PKCS#11 operation latency (target p99 < 100ms).

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `tlog_ingest_lag_high` | Kafka consumer lag > 1000 | warning |
| `tlog_sth_signing_stalled` | No new STH in > 10s | critical |
| `tlog_channel_publish_failed` | Channel publish failure for > 180s | warning |
| `tlog_hsm_unavailable` | PKCS#11 ping fails | critical |
| `tlog_witness_cosign_missing` | No IAB cosig in > 3600s on election day | warning |

### Key metrics

- `eel_tlog_leaves_total` - Counter: total leaves appended.
- `eel_tlog_sth_signing_duration_seconds` - Histogram: HSM sign latency.
- `eel_tlog_channel_publish_lag_seconds` - Gauge per channel.
- `eel_tlog_tree_size` - Gauge: current tree size.
- `eel_hsm_pkcs11_op_duration_seconds` - Histogram: per key-role.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | `cargo test --lib` | Merkle tile computation, JCS canonicalization, inclusion proof construction, STH serialization |
| Property | proptest / quickcheck | Append-then-prove round-trip: any leaf hash produces a valid proof against the current root |
| Contract | OpenAPI conformance (`cargo test --test '*_contract'`) | REST response shapes match OpenAPI 3.1 spec; CloudEvent envelopes match schema |
| Integration | `cargo nextest run` with testcontainers (PostgreSQL, Kafka) | Full ingest-to-STH pipeline; five-channel publish to stub channels; PKCS#11 software token |
| Load | k6 (`loadtest/`) | 500 leaf appends/s for 600s; STH latency p99 < 1s; proof serving p99 < 200ms |
| Chaos | planned (Wave 3) | HSM sidecar kill; Kafka partition loss; PG primary failover |
| Formal | TLA+ / Apalache (planned, docs/tla if present) | Append monotonicity; consistency proof soundness |
| Security | PKCS#11 fuzzer (planned) | Malformed key operation requests to signer sidecar |

---

## 14. Follow-ups and known limitations

- `disclosures` and `repro_bundles` tables are defined in README and data_dictionary.yaml but migration files for those specific tables were not found in `migrations/` at the time of writing. Verify in Wave 2 migration pass.
- Campaign finance forms compliance rules (finance-transparency crate) are planned. Crate directory exists (`crates/finance-transparency`) but no Rust source files were found on disk. Marked in-flight for Wave 2.
- Gazette print anchor and KBC radio adapters are planned. Only HTTP/GitHub/IPFS channels have concrete adapter code in `crates/tlog-witness`.
- ML-DSA (PQC) hybrid signing is implemented in `crates/hsm-signer` per README, but requires an HSM with CRYSTALS-Dilithium support (e.g., Thales Luna Network HSM 7.4+). Verify HSM model before production deployment.
- Formal TLA+ specs referenced in `docs/tla` directory (boundary-svc and rts-svc have a `tla` subdirectory; transparency-svc does not). Consistency proof spec deferred to Wave 3.

---

## Appendix A: Merkle tile tree mechanics

The transparency log uses a tile-based Merkle tree layout (RFC 6962-bis tile terminology). Tiles are fixed-width blocks of 256 leaf hashes or 256 internal node hashes. This contrasts with a naive binary tree where each proof path requires O(log N) individual hash reads.

```
Level 0 (leaf tiles):   [ tile_0_0 | tile_0_1 | tile_0_2 | ... ]  <- 256 leaves each
Level 1 (node tiles):   [ tile_1_0 | tile_1_1 | ... ]              <- 256 level-0 tile roots each
Level 2 (node tiles):   [ tile_2_0 | ... ]                         <- root approaches
```

Tile cache stored in `tree_tiles(level, tile_index, hash)`. An inclusion proof for leaf index `i` requires at most `ceil(log_256(N))` tile reads from the cache, typically 2-3 reads for a 22M-leaf tree.

Consistency proofs between tree sizes `n1` and `n2` walk the tile boundaries at each level, returning only the hashes of tiles that changed between the two tree sizes. The client can verify the proof by recomputing the root hash from the returned tiles and checking it matches the STH signature.

### STH cadences and publication channels

The transparency log publishes at three cadences:
- **1s cadence**: Optimistic STH covering the latest sequenced batch. Signed immediately by HSM. Published to IEBC website within 1s. Used by election-night real-time audit tools.
- **60s cadence**: Consolidated STH covering all 1s-STHs since the last 60s-STH. Preferred for consistency proof verification. Published to GitHub and IPFS-KE within 60s.
- **5min cadence**: Long-lived STH for archival and Gazette reference. Submitted to Gazette for next print run; announced on KBC Radio hash bulletin.

Each higher cadence STH has a consistency proof to the previous STH, so any verifier can walk from any historical 1s-STH to any current 60s-STH without re-downloading all leaves.

---

## Appendix B: Key ceremony protocol

The M-of-N key ceremony for the per-election-cycle KEK follows this protocol:

1. **Pre-ceremony**: Each keyholder receives their PKCS#11 smart card and PIN by separate couriers at least 7 days before ceremony date.
2. **Ceremony location**: Secure room at Commission headquarters with Faraday shielding; no mobile devices.
3. **Quorum assembly**: Minimum 3 of 5 keyholders physically present; identity verified by biometric + NID.
4. **Key generation**: HSM generates a fresh P-384 ECDH key pair. The private key is split into 5 Shamir shares (3-of-5 threshold) using the HSM's built-in key-splitting function.
5. **Share distribution**: Each share is encrypted to the keyholder's PKCS#11 card public key and written to the card. No share leaves the HSM in plaintext.
6. **Ceremony log**: All steps recorded in the meta-events log domain of the transparency log with HSM signatures. Ceremony log leaf hashes are announced on KBC Radio.
7. **KEK activation**: Once minimum quorum confirms their shares are readable, the KEK is activated. New voter and result data encrypted with DEKs wrapped under this KEK.
8. **Break-glass procedure**: If HSM fails during election, 3 keyholders reconvene; shares reconstructed under emergency HSM; KEK re-wrapped. RTO target: 30 min.

This procedure satisfies ISO/IEC 19790 Level 3 requirements for key ceremony documentation and quorum.

---

## Appendix C: Five-channel STH publication implementation notes

| Channel | Implementation | SLA | Failure mode |
|---------|---------------|-----|-------------|
| IEBC website | HTTPS PUT to CMS API; CDN invalidation | < 1s | CMS unavailable: retry with backoff; alert after 3 failures |
| GitHub | GitHub REST API v3 `PUT /repos/{owner}/{repo}/contents/{path}`; creates/updates a file with the STH JSON | < 60s | Rate limit or API outage: exponential backoff; secondary GitHub org as fallback |
| IPFS-KE | IPFS HTTP API v1 `POST /api/v0/add`; pin via pinning service | < 5 min | IPFS node unavailable: retry; secondary pinning service |
| Kenya Gazette | Email attachment to KAPI (Kenya Printers); hash read in next print issue | next business day | Email delivery failure: SFTP fallback; alert |
| KBC Radio | Automated script emails hash bulletin to KBC Radio newsroom for on-air reading | daily (18:00 EAT) | Email failure: fallback to SMS to newsroom |

All five channels provide independent, non-repudiable public evidence of the STH at a given point in time. A verifier that observes the STH on any two independent channels can be confident it has not been altered.

---

## Appendix D: Witness cosigning protocol

The witness cosigning protocol ensures that the Commission cannot unilaterally alter the transparency log without detection by independent witnesses.

- **IAB (Independent Administrative Board)**: Day-1 internal witness. Runs the `tlog-witness` client connecting to the witness gRPC endpoint. Cosigns every 60s-cadence STH within 30 minutes of publication. The `witness_cosigs.is_internal=true` flag marks IAB signatures.
- **External observers**: Day-2 external witnesses (civil society, political parties, international observers). Connect via the public `GET /v1/tlog/sth` endpoint and independently publish cosignatures. The `witness_cosigs.is_internal=false` flag marks external signatures.
- **Cosignature format**: Witnesses sign `Ed25519(concat(STH.tree_size_le64, STH.root_hash))` following the cosign bundle format (OpenSSF cosign). The signature is submitted via `POST /v1/tlog/witness/cosig` (authenticated with witness SPIRE SVID or API key).
- **Verifier guarantee**: Any third party can retrieve the STH, the witness cosignatures, and independently verify that at least N independent parties have confirmed the STH at that point in time. If the Commission attempts to fork the log (present different STHs to different clients), the inconsistency will be detected when any two clients compare their STHs and find they are not consistent (per the Merkle consistency proof).
