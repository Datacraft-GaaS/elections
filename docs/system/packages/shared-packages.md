# Shared Packages Reference (`packages/`)

Reference documentation for the shared libraries under `packages/`. All paths are relative to the repository root; line numbers refer to current source.

The `packages/` tree contains roughly thirty libraries in three language tiers (Rust, Python, Elixir), plus language-neutral assets (SpiceDB schemas, JSON event schemas, OpenAPI contracts, design tokens, locale files, data dictionary). Rust crates are workspace members via the glob `packages/eel-*-rs` in the root [Cargo.toml](../../../Cargo.toml). Python packages are `uv`/`pip`-installable projects under `packages/<name>/src/`; each also vendors a `.venv` for CI. Elixir libraries are Mix projects under `packages/<name>/lib/`.

> Note on consumers: consumer counts below were determined by grepping `services/*/Cargo.toml`, `services/**/pyproject.toml` and `services/**/mix.exs`. A notable finding: **no Elixir service `mix.exs` currently declares a dependency on any `eel-*-ex` package** (checked all eight `services/**/ex/*/mix.exs` files); the `eel-*-ex` libraries exist and are tested but are not yet wired into services. See "Known gaps" at the end.

---

## 1. Rust packages (`packages/eel-*-rs`)

### 1.1 `eel-crypto-rs` — cryptographic primitives

**Purpose.** Implements the cryptographic layer described in ADR-0005 ([packages/eel-crypto-rs/src/lib.rs:1](../../../packages/eel-crypto-rs/src/lib.rs)): Ed25519 signatures, BLAKE3 hashing, RFC 8785 JSON canonicalization, RFC 6962 Merkle trees, constant-time comparison, plus feature-gated PKCS#11 session pooling, post-quantum hybrid primitives, RFC 3161 timestamping and cosign verification.

**Module layout** (`packages/eel-crypto-rs/src/lib.rs:21-38`): always-compiled modules are `blake3`, `constant_time`, `ed25519`, `error`, `merkle`, `rfc8785`; feature-gated: `pkcs11_pool` (`pkcs11`), `pq` (`pq-hybrid` / `pq-slh` / `pq-kem`), `tsa` (`tsa`), `cosign` (`cosign`).

#### Merkle trees (`packages/eel-crypto-rs/src/merkle.rs`)

RFC 6962 Certificate Transparency primitives with SHA-256 domain separation: leaf hash is `SHA256(0x00 || data)`, node hash is `SHA256(0x01 || left || right)` ([merkle.rs:1-8](../../../packages/eel-crypto-rs/src/merkle.rs)).

| API | Location | Description |
|---|---|---|
| `type Hash = [u8; 32]` | merkle.rs:25 | SHA-256 hash value |
| `leaf_hash(data)` | merkle.rs:40 | RFC 6962 leaf hash |
| `node_hash(left, right)` | merkle.rs:58 | RFC 6962 internal node hash |
| `MerkleTree::build(entries: &[&[u8]])` | merkle.rs:101 | Build from raw leaf data; empty input yields root `SHA256("")` |
| `MerkleTree::from_leaf_hashes(leaves: Vec<Hash>)` | merkle.rs:128 | Build from pre-computed leaf hashes — entries are **not** re-hashed; used when callers hold domain-defined leaves (e.g. transparency-log rows in Postgres) |
| `size()` | merkle.rs:135 | Leaf count |
| `root()` | merkle.rs:149 | Tree head hash |
| `inclusion_proof(index)` | merkle.rs:175 | Sibling-hash path, leaf-to-root order; `CryptoError::InvalidMerkleProof` when out of range |
| `verify_inclusion(index, data, proof)` | merkle.rs:200 | Recomputes root from raw data; comparison uses `constant_time::ct_eq` (merkle.rs:212) |
| `verify_inclusion_against_root(root, index, tree_size, data, proof)` | merkle.rs:228 | Stateless verification against a known signed tree head |
| `verify_inclusion_against_root_leaf(root, index, tree_size, leaf, proof)` | merkle.rs:258 | Same, but starting from a pre-computed leaf hash (transparency-log verifiers holding only the leaf hash) |
| `consistency_proof(old_size)` | merkle.rs:301 | Proof that an old tree of `old_size` leaves is a prefix of this tree; empty proof for `old_size == 0` or equal sizes |
| `verify_consistency(old_root, old_size, proof)` | merkle.rs:320 | Constant-time root comparisons (merkle.rs:540-548) |

Internals: recursive RFC 6962 decomposition by `largest_power_of_two_less_than` (merkle.rs:483); odd trailing nodes are promoted unhashed per RFC 6962 §2 (merkle.rs:362); `recompute_root_from_leaf` (merkle.rs:418) replays proofs leaf-to-root and rejects both short (merkle.rs:451) and long (merkle.rs:462) proofs.

#### Other modules

| Module | Key API | Notes |
|---|---|---|
| `rfc8785` | `canonicalize(&Value) -> Result<Vec<u8>>` ([rfc8785.rs:44](../../../packages/eel-crypto-rs/src/rfc8785.rs)) | JCS implemented from scratch: UTF-16 code-unit key ordering (`cmp_utf16`, rfc8785.rs:97), ES2020 number serialization, rejects non-finite numbers (rfc8785.rs:118) |
| `constant_time` | `ct_eq(a,b)` (constant_time.rs:32), `ct_eq_fixed::<N>` (constant_time.rs:51) | Branch-free byte comparison used by every verification path above |
| `ed25519` | `generate_keypair` (:29), `sign` (:47), `verify` (:65), `verifying_key_from_bytes` (:86), `signature_from_bytes` (:102) | Pure-Rust Ed25519 |
| `blake3` | `hash` (blake3.rs:24), `keyed_hash` (:40), `hash_hex` (:53) | BLAKE3 hashing |
| `pq` | Hybrid Ed25519+ML-DSA signing (`HybridKeyPair`, pq.rs:40-115); ML-KEM-768 + X25519 KEM (`generate_keypair` :144, `encapsulate` :151, `decapsulate` :167); SLH-DSA archive sign/verify (:199, :212) | Feature-gated; requires liboqs |
| `pkcs11_pool` | `PkcsPool::open` (:82), `PkcsPool::get` (:126), `find_key_by_role` (:188) | Pooled PKCS#11 sessions against the HSM |
| `cosign` | `CosignBundle` (:40), `verify_bundle` (:54) | Sigstore/cosign bundle verification |
| `tsa` | `TsaConfig` (:34), `TsaClient::new` (:59) | RFC 3161 timestamp authority client |

`KeyRole` ([lib.rs:58-84](../../../packages/eel-crypto-rs/src/lib.rs)) enumerates HSM-managed key purposes and their labels: `Sth` → `eel-sth`, `Declaration` → `eel-declaration`, `KitCa`, `CredentialCa`, `CodeSigning`, `CycleKek`.

**Consumers.** 29 service crate manifests reference `eel-crypto-rs` (widest-used crate in the workspace).

**Security-relevant behaviors.**
- All Merkle root comparisons go through `constant_time::ct_eq` to prevent timing oracles.
- RFC 8785 canonicalization guarantees deterministic bytes before signing; non-finite numbers are hard errors.
- Proof length validation ("proof too short"/"too long", merkle.rs:461-463, 535-539) prevents malleable/truncated-proof acceptance.
- `from_leaf_hashes` deliberately does not re-hash, so verification semantics match exactly what a database-stored leaf claims.

**Tests.** Unit tests + proptest in merkle.rs:604-707 (inclusion/consistency round-trips, tampered-data rejection); Kani formal harnesses at merkle.rs:714-763 (`merkle_inclusion_proof_verifies_authentic`, `merkle_consistency_never_forgets`); doctests throughout.

---

### 1.2 `eel-hlc-rs` — Hybrid Logical Clock

**Purpose.** Hybrid Logical Clock timestamps (physical ms + counter + node id) providing causal ordering across nodes, per the HLC algorithm documented inline ([packages/eel-hlc-rs/src/lib.rs:200-289](../../../packages/eel-hlc-rs/src/lib.rs)).

**Public API surface.**

| Item | Location | Description |
|---|---|---|
| `HlcError` | lib.rs:31 | Parse errors |
| `trait WallClock` | lib.rs:53 | Injectable wall clock (`Send + Sync`) |
| `SystemWallClock` | lib.rs:67 | Default Unix-epoch-ms clock |
| `struct Hlc { physical_ms: u64, counter: u16, node: u16 }` | lib.rs:94 | Wire format `"physical_ms.counter.node"` string (Display/FromStr, lib.rs:104-140) |
| `HlcClock::new(node, wall)` | lib.rs:210 | Per-node clock |
| `now()` | lib.rs:222 | Strictly monotonic timestamp; bumps counter when physical time does not advance |
| `merge(incoming)` | lib.rs:256 | Causally-after merge implementing the published HLC update rules (comment block at lib.rs:266-276) |

**Consumers.** 8 service crate manifests (e.g. `services/rts-svc/crates/ballot-serial`, `services/rts-svc/crates/rts-tally`, `services/support-admin-svc/crates/special-voter`, `services/field-mobile-svc`).

**Security-relevant behaviors.** Strict monotonicity guarantee (lib.rs:220); counter overflow handled with `saturating_add`. The kit SDK's duress detection compares server HLCs against device clocks (see [kit-sdk-flutter](#17-non-code-and-multi-language-packages)); clock manipulation > 5 minutes raises a suspect signal there.

**Tests.** Doctests; Kani harnesses in lib.rs (`kani_harnesses` module after line 293) verifying monotonicity and merge properties.

---

### 1.3 Events triplet (`eel-events-rs`, `eel-events-py`, `eel-events-ex`)

**Shared semantics.** CloudEvents 1.0 envelope with mandatory `eel_*` platform extensions (`eel_stream_id`, `eel_seq`, `eel_hlc`, `eel_prev_hash`, `eel_correlation_id`, `eel_causation_id`, `eel_producer_key_id`, `eel_signature`, `eel_election_id`, `eel_data_class`). Signing is over RFC 8785-canonicalized envelope bytes with the `eel_signature` field removed; signature format is `ed25519:<base64url>`. Event type naming: `eel.<context>.<aggregate>_<event_name>.v<n>`. Data classification follows ADR-0016 (classes A–E). Schemas live in [packages/event-schemas](#18-language-neutral-packages) and register in Karapace.

#### Rust: `eel-events-rs`

Entry file: [packages/eel-events-rs/src/lib.rs](../../../packages/eel-events-rs/src/lib.rs) (module list at lib.rs:26-43).

| Item | Location | Description |
|---|---|---|
| `enum DataClass { A..E }` | envelope.rs:7 | A = most sensitive; default B (envelope.rs:15) |
| `struct Envelope` | envelope.rs:27 | CloudEvents 1.0 + eel extensions |
| `Envelope::canonical_bytes_for_signing()` | envelope.rs:101 | Strips `eel_signature`, applies `eel_crypto_rs::rfc8785::canonicalize` |
| `EventBuilder` | builder.rs:18 | Fluent builder: `event_type` (:39), `source` (:44), `subject` (:49), `data` (:59), `stream_id` (:64), `seq` (:69), `hlc` (:74), `prev_hash` (:79), `correlation_id`/`causation_id` (:84/:89), `election_id` (:94), `data_class` (:99) |
| `build_unsigned()` | builder.rs:106 | Validate without signing |
| `build(key_provider)` | builder.rs:136 | Sign via `SpiffeKeyProvider` |
| `trait SpiffeKeyProvider` | spiffe.rs:8 | `signing_key()` + `key_id()` (SPIFFE SVID fingerprint); implementations rotate hourly |
| `sign_leaf` / `verify_leaf` / `verify_classical` | pq_leaf.rs:187/:215/:244 | ML-DSA transparency-log leaf signing with classical fallback |
| `ChainedAudit` trait / `AuditEntry::compute_hash` | audit_chain.rs:92 / :54 | Hash-chained audit trail; `ChainVerifier::verify_all` (audit_chain.rs:188) |
| `consumer::verify_signature(envelope, provider)` | consumer.rs:30 | Consumer-side signature check |
| `GapBuffer` | consumer.rs:68 | Reorders out-of-order events by `eel_seq` |
| `DedupeStore::check_and_record` | consumer.rs:146 | Postgres-backed exactly-once dedupe |
| `dispatch<C: EventConsumer>` | consumer.rs:190 | Signature verify → dedupe → gap-buffer → handler pipeline |
| `dlq_topic` / `tamper_topic` | consumer.rs:118/:123 | Dead-letter and tamper-quarantine topic naming |
| `KafkaConsumerRunner` | consumer.rs:250 | Feature-gated rdkafka loop |
| `KarapaceClient::get_schema` | karapace.rs:43 | TTL-cached schema registry client |

**Security-relevant behaviors:** producer-side signing keyed by SPIFFE identity; tamper quarantine topic for failed signature checks; hash-chained audit entries with self-consistency check (`AuditEntry::is_self_consistent`, audit_chain.rs:64).

**Tests.** `tests/envelope_tests.rs`, `tests/audit_chain_test.rs`.

#### Python: `eel-events-py`

Entry package: [packages/eel-events-py/src/eel_events/](../../../packages/eel-events-py/src/eel_events/).

- `envelope.Envelope` Pydantic model with `DataClass` enum (envelope.py:14, :22) and `canonical_bytes()` (envelope.py:46).
- `rfc8785.canonicalize(obj) -> bytes` (rfc8785.py:10) — Python reimplementation.
- `signing.sign_envelope(envelope, private_key)` (signing.py:23) returns `"ed25519:<base64url>"`; `verify_envelope(envelope, public_key)` (signing.py:40).
- `builder.EventBuilder` mirrors the Rust builder (`build_unsigned` builder.py:76, async `build(key_provider)` :83).
- `karapace.KarapaceClient` with `get_schema` / `register_schema` / `check_compatibility` (karapace.py:20-44).
- `consumer.ConsumerMiddleware` (consumer.py:31) with `DedupStore` protocol (:19).

**Difference vs Rust:** Python lacks the `pq_leaf`, `audit_chain`, and Kafka runner modules; it exposes schema registration/compatibility checks that the Rust Karapace client does not.

**Tests.** `tests/ci/test_events.py`.

#### Elixir: `eel-events-ex`

Entry module: [packages/eel-events-ex/lib/eel/events.ex](../../../packages/eel-events-ex/lib/eel/events.ex).

| Item | Location | Description |
|---|---|---|
| `Eel.Events.Envelope` struct | envelope.rs analogue: envelope.ex:1 | `from_map` (:83), `to_map` (:107), `validate` (:122) |
| `Eel.Events.Builder` | builder.ex:1 | `new/3` (:53), `with_stream/4` (:63), `with_data_class/2` (:84), `with_hlc/2` (:89), `build/1` (:96) |
| `Eel.Events.CanonicalJson.encode/1` | canonical_json.ex:19 | RFC 8785-style encoding |
| `Eel.Events.Signing.sign/2` / `verify/2` | signing.ex:24 / :42 | Ed25519 over canonical bytes, PEM decode helpers (:69, :92) |
| `Eel.Events.Consumer` behaviour | consumer.ex:30 | `handle_event/2` callback |
| `Eel.Events.Karapace.lookup_schema/2` | karapace.ex:37 | GenServer-cached registry lookups |

**Tests.** Under `packages/eel-events-ex/test/` (ExUnit).

---

### 1.4 Outbox triplet (`eel-outbox-rs`, `eel-outbox-ex`)

*(No `eel-outbox-py` exists; Python/Django surfaces use the `OutboxRecord` Django model in `eel-web-py`, see [1.13](#113-eel-web-py--django-shared-web-layer).)*

**Shared semantics.** Transactional outbox: events are inserted into a Postgres `outbox` table inside the caller's transaction; Debezium CDC publishes them atomically to Kafka, closing the dual-write race ([packages/eel-outbox-rs/src/lib.rs:1-12](../../../packages/eel-outbox-rs/src/lib.rs)). Payload limit 1 MiB in both implementations; large blobs offloaded to MinIO with a compact reference envelope.

#### Rust: `eel-outbox-rs`

| Item | Location | Description |
|---|---|---|
| `struct Outbox` | outbox.rs:31 | Wraps a `sqlx::PgPool` |
| `emit(tx, envelope)` | outbox.rs:57 | Insert into caller's transaction; rejects payloads > 1 MiB (`PayloadTooLarge`); builds jsonb headers from `eel_*` extensions |
| `migrate()` | outbox.rs:127 | Creates the outbox table |
| `struct OutboxEnvelope` | envelope.rs:14 | Full CloudEvent + routing fields (`topic`, `kafka_key`, aggregate ids) |
| `to_json_bytes` / `signature_bytes` | envelope.rs:64 / :69 | Serialization helpers |
| `upload_blob(config, ...)` | minio.rs:37 | MinIO blob offload (`MinioConfig` minio.rs:10, `BlobRef` :23) |

**Consumers.** 24 service crate manifests.

**Security-relevant behaviors.** Sole sanctioned path to Kafka — direct produces are forbidden by convention; payload size guard prevents oversized-message DoS; envelope carries `eel_data_class` so downstream consumers can enforce handling tiers.

**Tests.** `tests/unit_tests.rs`, `tests/integration_roundtrip.rs`.

#### Elixir: `eel-outbox-ex`

Entry: [packages/eel-outbox-ex/lib/eel/outbox.ex](../../../packages/eel-outbox-ex/lib/eel/outbox.ex). `Eel.Outbox.emit/2` (outbox.ex:39) appends into an `Ecto.Multi`; returns `{:error, :payload_too_large}` beyond 1 MiB (`@max_payload_bytes` outbox.ex:15). `Eel.Outbox.Blob.maybe_offload/2` handles MinIO offload (blob.ex); `Eel.Outbox.Entry` is the Ecto schema (entry.ex); `Eel.Outbox.Checks.NoRawKafka` (checks/no_raw_kafka.ex) is a compile-time/CI check enforcing that no code writes to Kafka directly. No mix.exs consumers found in `services/` (not verified as used).

---

### 1.5 Event store triplet (`eel-eventstore-rs`, `eel-eventstore-py`, `eel-eventstore-ex`)

**Shared semantics.** Append-only event sourcing on Postgres tables `event_streams` and `snapshots`, shared across all three languages (same schema, bidirectional interoperability per READMEs). Optimistic concurrency via expected-version guards. **Documented limitation (all three READMEs): snapshot compaction (deleting old events before a snapshot) is not implemented.**

#### Rust: `eel-eventstore-rs`

README: [packages/eel-eventstore-rs/README.md](../../../packages/eel-eventstore-rs/README.md) (Limitations section at README.md:104-106).

| Item | Location | Description |
|---|---|---|
| `struct EventStore` | store.rs:14 | Wraps `PgPool`; `create_schema_sql` (:24), `migrate` (:29) |
| `append_events(...)` | store.rs:41 | Write with optimistic-concurrency guard |
| `load_events(...)` | store.rs:94 | Stream read |
| `save_snapshot` / `load_snapshot` | store.rs:144 / :166 | Snapshot persistence (JSON format) |
| `replay<A, F>(...)` | store.rs:191 | Full-stream fold into aggregate state |
| `StoredEvent` / `AggregateSnapshot` / `Aggregate` trait | aggregate.rs:9 / :22 / :33 | Domain contract (`apply`, `snapshot`, `initial_state`) |
| `EventReplay` / `SnapshotTailReplay` | replay.rs:11 / :49 | Async iteration; snapshot-first rebuild (`rebuild`, replay.rs:63) |
| `ExactlyOnceConfig` / `ExactlyOnceConsumer` | exactly_once.rs:18 / :27 | `kafka` feature: transactional consumer/producer pair; offsets committed atomically with produced messages (`init_transactions` at exactly_once.rs:66) |

**Consumers.** 10 service crate manifests.

**Tests.** `tests/proptest_replay.rs` (property-based replay equivalence); benchmark `benches/replay_1m.rs`.

#### Python: `eel-eventstore-py`

Entry: [packages/eel-eventstore-py/src/eel_eventstore/](../../../packages/eel-eventstore-py/src/eel_eventstore/). `EventStore` class (store.py:52) over SQLAlchemy `AsyncSession`: `append_events` (:58), `load_events` (:109), `save_snapshot` (:141), `load_snapshot` (:161), `replay` (:183). `OptimisticConcurrencyError` (store.py:40). Pydantic models `StoredEvent` / `AggregateSnapshot` (models.py:10/:21), `Aggregate` protocol (protocols.py:10). Limitations (README.md:87-89): requires `asyncpg`; no compaction; use `replay_from_snapshot()` for >100k-event streams. Test: `tests/ci/test_eventstore.py`.

#### Elixir: `eel-eventstore-ex`

Entry: [packages/eel-eventstore-ex/lib/eel/event_store.ex](../../../packages/eel-eventstore-ex/lib/eel/event_store.ex) — delegates to `append_events(repo, stream_id, events, expected_version)` (event_store.ex:17 → stream.ex:20) and `replay(repo, stream_id, aggregate_module, opts)` (event_store.ex:27 → replay.ex:24). Supporting modules: `Stream` (optimistic append returning `{:error, :version_conflict}`), `Replay` (snapshot-aware), `Aggregate` behaviour with `apply/2`, `snapshot/1`, `initial_state/0` callbacks (aggregate.ex:12-22), `Snapshot` Ecto schema (snapshot.ex:1), `Event` Ecto schema (event.ex:1). No mix.exs consumers found in `services/`.

---

### 1.6 PII triplet (`eel-pii-rs`, `eel-pii-py`, `eel-pii-ex`)

**Shared semantics.** Purpose-bound access control for personal data per ADR-0016. Values are wrapped in typed newtypes whose tier is fixed statically (Tier A restricted … Tier E public). Reading a Tier-A/B value requires calling `reveal(purpose, audit_sink)`, which records the access. `Debug`/`Display`/serialization always redact, so raw values cannot leak into logs by accident.

#### Rust: `eel-pii-rs`

Single-file crate: [packages/eel-pii-rs/src/lib.rs](../../../packages/eel-pii-rs/src/lib.rs).

| Item | Location | Description |
|---|---|---|
| `enum Purpose` | lib.rs:39 | 12 statutory purposes: Enrolment, Identification, Adjudication, Publication, StatutoryReport, GrievanceIntake, SupportTicket, Audit, DataPortability (DPA s.26), ErasureRequest (DPA s.40), SecurityInvestigation, InternalMonitoring |
| `enum PIIDataClass { A..E }` | lib.rs:72 | ADR-0016 tiers |
| `trait Classified` | lib.rs:98 | Static `data_class()` declaration |
| `trait PIIAuditSink` + `NoopAuditSink` / `TracingAuditSink` | lib.rs:111 / :128 / :142 | Access-event receivers |
| `SerdeRedact` | lib.rs:172 | Serialize placeholder |
| `PIIRestricted<T>` | lib.rs:205 | Tier-A wrapper: `reveal` (:214) audits then returns `&T`; `into_inner` (:220); `as_redacted` (:226); `Debug` prints `PIIRestricted(***REDACTED***)` (lib.rs:250 area) |
| `PIISensitive<T>` | lib.rs:304 | Same shape for Tier-B |
| Macro-generated wrappers | lib.rs:422 | Lower-tier newtypes |

**Compile-time enforcement.** `tests/compile_fail/no_reveal_without_purpose.rs` — a trybuild test proving `.reveal()` cannot be called without a `Purpose`.

**Consumers.** 5 service crate manifests (`services/bvr-svc/*`, `services/vprs-svc/*`).

#### Python: `eel-pii-py`

[packages/eel-pii-py/src/eel_pii/__init__.py](../../../packages/eel-pii-py/src/eel_pii/__init__.py): `Purpose` enum (:34), `PIITier` (:43), `PIIAuditSink` protocol (:51), `_PIIBase` generic wrapper (:62) with redacting `__repr__`/`__str__` (:71/:74) and `reveal(purpose, sink)` (:77), Pydantic core-schema integration (:87) so wrapped values validate inside Pydantic models. Concrete classes `PIIRestricted` (:111), `PIISensitive` (:118), `PIIOperational` (:125). Test: `tests/ci/test_pii.py`.

**Difference vs Rust:** adds `PIIOperational`; integrates with Pydantic rather than serde; reveal is optional-sink (`None` allowed) whereas Rust requires a sink reference.

#### Elixir: `eel-pii-ex`

[packages/eel-pii-ex/lib/eel/pii.ex](../../../packages/eel-pii-ex/lib/eel/pii.ex): structs `%Restricted{}` (:22), `%Sensitive{}` (:30), `%Public{}` (:38); constructors `restricted/2` (:52, classes A/B), `sensitive/2` (:58, C/D), `public/1` (:64); `reveal/2` (:79-89); `class/1` (:96); `redact/1` (:102). `Eel.PII.EctoType.Restricted` (ecto_types.ex:2) is an Ecto custom type whose `dump/1` returns `{:error, :pii_dump_forbidden}` (:28) — restricted PII can be loaded but never persisted back through this type. `Eel.PII.Protocols` (protocols.ex:1) provides redacting inspect protocols. No mix.exs consumers found in `services/`.

---

### 1.7 Resilience triplet (`eel-resilience-rs`, `-py`, `-ex`)

**Shared semantics.** Standard resilience vocabulary across all tiers: timeout, retry-with-full-jitter exponential backoff, circuit breaker (closed/open/half-open), bulkhead concurrency limits, degradation flag (`normal` / `degraded` / `read_only`), and idempotency-key storage/middleware.

#### Rust: `eel-resilience-rs`

| Item | Location | Description |
|---|---|---|
| `with_timeout(duration, f)` | timeout.rs:7 | Futures timeout → `ResilienceError::Timeout` |
| `RetryConfig` / `JitterKind` | retry.rs:16 / :7 | Defaults: 3 attempts, 100ms→30s, full jitter |
| `with_retry(config, should_retry, f)` | retry.rs:37 | Caller decides which errors are retryable (idempotent ops only) |
| `CircuitBreaker::new` / `allow` / `record_success` / `record_failure` / `state` | circuit_breaker.rs:41 / :64 / :82 / :92 / :109 | Shared-state breaker |
| `Bulkhead::acquire` | bulkhead.rs:21 | Permit-based concurrency cap per dependency |
| `DegradationFlag` | degradation.rs:25 | `get`/`set`/`is_write_allowed` (:36/:40/:44) |
| `IdempotencyStore::check` / `store_response` | idempotency.rs:42 / :71 | Body-hash keyed response cache (`body_hash` :33) |

Tests: `tests/resilience_test.rs`.

#### Python: `eel-resilience-py`

[packages/eel-resilience-py/src/eel_resilience/](../../../packages/eel-resilience-py/src/eel_resilience/): `with_timeout(coro, timeout_s)` (timeout.py:10), `retry_with_jitter` (retry.py:39), `CircuitBreaker` (circuit_breaker.py:24), `Bulkhead` (bulkhead.py:16), `DegradationFlag` (degradation.py:20), `IdempotencyMiddleware` Starlette middleware (idempotency.py:47) with `InMemoryIdempotencyStore` (:28) behind the `IdempotencyStore` protocol (:14). Test: `tests/ci/test_resilience.py`.

#### Elixir: `eel-resilience-ex`

[packages/eel-resilience-ex/lib/eel/resilience/](../../../packages/eel-resilience-ex/lib/eel/resilience/): `Timeout.with_timeout/2` (timeout.ex:13), `Retry.with_retry/2` (retry.ex:19), `CircuitBreaker` GenServer with `call/3` and `state/1` (circuit_breaker.ex:50/:58), `Bulkhead.checkout/3` (bulkhead.ex:39), `DegradationFlag` persistent_term flag with `set/get/degraded?/read_only?` (degradation_flag.ex:16-41), `IdempotencyPlug` Plug router plug (idempotency_plug.ex:25), plus a `ResilienceRequired` action-check (checks/resilience_required.ex). No mix.exs consumers found in `services/`.

---

### 1.8 Observability triplet (`eel-observability-rs`, `-py`, `-ex`)

**Shared semantics.** OpenTelemetry initialization from env, log scrubbing for PII markers, SLO recording-rule definitions, and SQL generation for "audit shadow" triggers (database-side copies of sensitive-row changes feeding the audit pipeline; driven by [packages/data-dictionary](#18-language-neutral-packages)).

#### Rust: `eel-observability-rs`

| Item | Location | Description |
|---|---|---|
| `OtelConfig::from_env(service_name)` / `init_otel` | otel.rs:20 / otel.rs:47 | OTLP exporter setup, returns `OtelGuard` |
| `PiiScrubberLayer` | scrubber.rs:11 | tracing layer warning when `***REDACTED***` markers appear in log fields — signals a PII-typed value reached a log macro (ADR-0016 violation); see scrubber.rs:1-9 rationale |
| `SloDefinition::new(service, indicator, target)` | slo.rs:17 | Auto-generates Prometheus recording rule names |
| `audit_shadow_sql(table_name)` | audit_shadow.rs:8 | Generates trigger DDL mirroring row changes into shadow tables |
| `install_audit_shadow(...)` | audit_shadow.rs:51 | Applies the DDL |

Note: **no service Cargo.toml consumes `eel-observability-rs` directly** (grep returned zero matches); observability wiring may occur via `eel-api-server-rs` or service-local setup — not verified.

Tests: `tests/observability_test.rs`.

#### Python: `eel-observability-py`

[packages/eel-observability-py/src/eel_observability/](../../../packages/eel-observability-py/src/eel_observability/): `init_otel` (otel.py:11), `get_tracer`/`get_meter` (:38/:42), `PiiScrubberFilter` logging filter + `install_scrubber` (log_scrubber.py:23/:36), SLO helpers `record_latency/record_error/record_success` + `SloTimer` (slo.py:9-50), `generate_audit_shadow_trigger(table_name, schema)` (audit_shadow.py:46).

#### Elixir: `eel-observability-ex`

[packages/eel-observability-ex/lib/eel/observability.ex](../../../packages/eel-observability-ex/lib/eel/observability.ex): `setup/1` (:18), `start_span/3` (:28), `add_event/2` (:38), `record_error/1` (:47); companion modules `LogScrubber` (log_scrubber.ex:1), `AuditShadow` (audit_shadow.ex:1), `Slo` (slo.ex:1). No mix.exs consumers found in `services/`.

---

### 1.9 Auth triplet (`eel-auth-rs`, `eel-auth-py`, `eel-auth-ex`)

**Shared semantics.** JWT verification against Keycloak JWKS (RS256), extraction of a `Subject` (identity + roles from realm claim), SpiceDB fine-grained authorization checks, and mandatory audit logging of every authorization decision.

#### Rust: `eel-auth-rs`

| Item | Location | Description |
|---|---|---|
| `Subject` | subject.rs:5 | Verified principal |
| `JwksVerifier::new(jwks_url, realm)` / `verify(token)` | jwks.rs:52 / :93 | Cached keys (TTL 300 s, capacity 64, jwks.rs:55-59); kid-miss refetch (jwks.rs:80-88); issuer validation derived from JWKS URL; audience validation disabled (`validate_aud = false`, jwks.rs:107 — noted as a deliberate choice, reviewers should confirm threat model) |
| `Consistency` enum | spicedb.rs:9 | SpiceDB request consistency modes |
| `Decision` | spicedb.rs:20 | Allow/deny + metadata |
| `SpiceDbClient::check(...)` | spicedb.rs:53 | Permission check wired to an `AuditSink` |
| `invalidate_cache()` | spicedb.rs:161 | Cache flush |
| `AuditEvent::new(subject, action, resource, decision, metadata)` | audit.rs:20 | Decision record |
| `AuditSink` trait (+ `NoopSink`, `LogSink`) | audit.rs:34 / :38 / :47 | Sink abstraction |

Tests: `tests/integration_test.rs`.

#### Python: `eel-auth-py`

[packages/eel-auth-py/src/eel_auth/](../../../packages/eel-auth-py/src/eel_auth/): `JwksCache.get_key/verify_token` (jwks.py:37/:77), `SubjectExtractor.extract` (subject.py:32, configurable roles claim default `realm_access.roles`), `SpiceDBClient.check/lookup_resources` (spicedb.py:41/:87), FastAPI `AuthMiddleware` (middleware.py:15) and `require_permission(permission, resource_type)` dependency factory (middleware.py:59), audit sinks (audit.py:10-51). Tests: `tests/ci/test_auth.py`.

#### Elixir: `eel-auth-ex`

[packages/eel-auth-ex/lib/eel/auth/](../../../packages/eel-auth-ex/lib/eel/auth/): `Eel.Auth.JWT.verify/2` (jwt.ex:20) against `Eel.Auth.JWKS` GenServer cache (`get_public_key/2`, jwks.ex:41), `Eel.Auth.SpiceDB.check_permission/5` and `lookup_resources/4` (spicedb.ex:21/:39), Plug integration `Eel.Auth.Plug` (plug.ex:31), LiveView `on_mount(:require_authenticated, ...)` hook (live_auth.ex:22), `Eel.Auth.Subject` struct (subject.ex:1). No mix.exs consumers found in `services/`.

---

### 1.10 Secrets triplet (`eel-secrets-rs`, `eel-secrets-py`, `eel-secrets-ex`)

**Shared semantics.** Integration with OpenBao/Vault-compatible secret management: transit encryption-as-a-service, dynamic short-lived Postgres credentials, JIT admin credentials, and AppRole auth (Python).

#### Rust: `eel-secrets-rs`

| Item | Location | Description |
|---|---|---|
| `Transit::encrypt(key_name, plaintext)` | transit.rs:28 | Returns base64 ciphertext string |
| `Transit::decrypt(key_name, ciphertext)` | transit.rs:60 | |
| `Transit::cycle_kek(cycle_id)` | transit.rs:96 | Rotates cycle KEK (matches `KeyRole::CycleKek` in eel-crypto-rs) |
| `issue_leaf_cert(...)` -> `LeafCert` | pki.rs:15 / :6 | PKI leaf issuance |
| `issue_admin_cred(...)` -> `AdminCred` | admin_creds.rs:20 / :10 | Just-in-time admin credentials |
| `PgDynamicCred::new(...) / pool()` | dynamic_creds.rs:28 / :44 | Dynamic DB credentials wrapped into a ready `PgPool` |

Tests: `tests/integration_test.rs`. Note: no direct service Cargo.toml references found (see 1.8 note).

#### Python: `eel-secrets-py`

[packages/eel-secrets-py/src/eel_secrets/](../../../packages/eel-secrets-py/src/eel_secrets/): `OpenBaoClient` with AppRole login (`auth_approle`, client.py:41), KV read/write (:82/:102); `Transit` encrypt/decrypt/rotate_key (transit.py:17/:37/:56); `PgDynamicCred.get/renew/revoke` + `connection_string` (dynamic_creds.py:33/:57/:85/:102).

#### Elixir: `eel-secrets-ex`

[packages/eel-secrets-ex/lib/eel/secrets.ex](../../../packages/eel-secrets-ex/lib/eel/secrets.ex): supervisor `child_spec/1` (:23), `read/2` (:40), `encrypt/3` (:51), `decrypt/3` (:69). `Eel.Secrets.JitAdminCred.fetch(role, ttl_hours \\ 4)` (jit_admin_cred.ex:16); `Eel.Secrets.PgDynamicCred` GenServer with `get_credential/1` and `renew/1` auto-refresh (pg_dynamic_cred.ex:45/:53). No mix.exs consumers found in `services/`.

---

### 1.11 Load shedding triplet (`eel-loadshedding-rs`, `-py`, `-ex`)

**Shared semantics.** CoDel-style queue-delay load shedding with four priority classes derived from request headers/auth state: anonymous requests shed first, admin requests last. Priority header: `X-Eel-Priority`.

#### Rust: `eel-loadshedding-rs`

| Item | Location | Description |
|---|---|---|
| `enum Priority` | priority.rs:7 | Anonymous=0 < AccreditedMachine=1 < AuthenticatedUser=2 < Admin=3; doc comment states admin requests are never shed first |
| `Priority::from_headers(headers)` | priority.rs:27 | `X-Eel-Priority` header or auth fallback |
| `CoDelConfig` / `CoDelShedder` | shedder.rs:8 / :45 | Queue-delay estimator |
| `admit(priority, request_start)` | shedder.rs:69 | Returns permit or shed count |
| `LoadSheddingLayer` / `LoadSheddingService<S>` | layer.rs:17 / :43 | tower layer/service |
| `ShedMetrics` | metrics.rs:6 | Shed counts, breaches, queue depth snapshots |

Tests: `tests/loadshedding_test.rs`.

#### Python: `eel-loadshedding-py`

[packages/eel-loadshedding-py/src/eel_loadshedding/middleware.py](../../../packages/eel-loadshedding-py/src/eel_loadshedding/middleware.py): `Priority` enum (:15), `shed_response(retry_after_s=5)` (:22, emits 503 + Retry-After), `default_priority_extractor` (:42), `LoadSheddingMiddleware` (:58).

#### Elixir: `eel-loadshedding-ex`

[packages/eel-loadshedding-ex/lib/eel/load_shedding/](../../../packages/eel-loadshedding-ex/lib/eel/load_shedding/): `Controller` GenServer — `should_shed?(priority)` (controller.ex:44), `report_latency(ms)` (:54) driving the CoDel state machine (handle_call logic :76-93); Plug front-end (plug.ex:33); top-level `child_spec/1` and `shed_request?/1` (load_shedding.ex:15/:27). No mix.exs consumers found in `services/`.

---

### 1.12 `eel-api-server-rs` — opinionated Axum server scaffold

**Purpose.** Composes the platform's HTTP conventions into one builder so Rust services get identical security middleware: JWT auth, idempotency, degradation write-guard, content negotiation (JSON/CBOR), request IDs, RFC 9457 problem responses, CoDel load shedding, and signed responses.

**Public API surface.**

| Item | Location | Description |
|---|---|---|
| `ApiServer::builder()` | server.rs:65 | Entry point |
| `ApiServerBuilder` setters | builder.rs:42-77 | `jwks_url`, `jwks_realm`, `otel_service_name`, `degradation_flag`, `idempotency_ttl_secs`, `codel_config`, `bind_addr`, `build` |
| `into_router(app_routes)` / `serve(app_routes)` | server.rs:72 / :95 | Mount domain routes under the standard middleware stack |
| `middleware::auth::require_jwt` | middleware/auth.rs:14 | JWT extraction/verification |
| `middleware::idempotency::idempotency_check` | middleware/idempotency.rs:8 | `Idempotency-Key` enforcement |
| `middleware::degradation_guard::guard_writes` | middleware/degradation_guard.rs:6 | Reject writes when `DegradationMode` forbids them |
| `middleware::content_negotiation::negotiate` | middleware/content_negotiation.rs:6 | JSON/CBOR negotiation (`ContentPrefs` :29) |
| `middleware::request_id::inject_request_id` | middleware/request_id.rs:6 | Adds `RequestId` (request_id.rs:23) |
| `Problem` / `ProblemType` | problem.rs:44 / :9 | RFC 9457 bodies; constructors `unauthorized` (:76), `forbidden` (:80), `write_locked` (:84, maps degraded mode), `idempotency_mismatch` (:92); `to_axum_response` (:100) |
| `SignedJson<T>::ok(payload)` / `with_snapshot(snapshot_id, leaf)` | response.rs:39 / :48 | Responses carrying `X-Eel-Signature` (`sign_body` response.rs:16) and Merkle proof headers (`merkle_proof_header` response.rs:26) |
| `health_router()` | routes.rs:4 | Standard health/readiness endpoints |

Tests: `tests/server_tests.rs`.

**Consumers.** `rts-api`, `rts-ro-api`, `rts-device-api`, `crms-api`, `field-mobile-svc` (per Cargo.toml grep).

**Security-relevant behaviors.** Centralizes authentication, idempotency, and write-lock enforcement so individual handlers cannot bypass them; signs Tier-A response bodies; problem-type URIs constrained to the registered set in [packages/openapi-contracts/_shared/problem-types.yaml](../../../packages/openapi-contracts/_shared/problem-types.yaml).

---

### 1.13 `eel-trace-rs` — traceability attribute macro

**Purpose.** `#[trace("EC-DAT-01", "FR-M02-007")]` no-op proc-macro attribute that carries requirement traceability IDs at test sites so `eel-completeness trace` can map tests to requirements ([packages/eel-trace-rs/src/lib.rs:1-23](../../../packages/eel-trace-rs/src/lib.rs)). Expansion is a pure pass-through — annotations never alter behavior. Consumers include `ecc-api`, `crms-domain`, RTS crates, and even the Tauri desktop app (`services/rts-svc/apps/ro-desktop/src-tauri`).

### 1.14 `verifier-rs` — public result verification

**Purpose.** Standalone verifier for the transparency log: RFC 6962 inclusion/consistency proofs, STH signature checks, witness cosignature aggregation, and election-bundle hash verification. Exposed three ways: Rust API, C ABI (`ffi.rs`, cdylib feature for Dart FFI), WASM exports ([packages/verifier-rs/src/wasm.rs:23-82](../../../packages/verifier-rs/src/wasm.rs)).

**Public API surface** ([packages/verifier-rs/src/lib.rs](../../../packages/verifier-rs/src/lib.rs)):

| Item | Location | Description |
|---|---|---|
| `Sth` | lib.rs:27 | Signed tree head: size, timestamp, hex root/signature/log-key |
| `InclusionProof` / `ConsistencyProof` | lib.rs:40 / :50 | Hex-encoded sibling paths |
| `WitnessSig` | lib.rs:59 | Ed25519 witness cosignature |
| `VerifierError` | lib.rs:72 | Typed errors incl. strict 32-byte hash decode (`decode_hash` lib.rs:105) |
| `verify_inclusion(...)` | lib.rs:220 | |
| `verify_consistency(...)` | lib.rs:252 | |
| `verify_sth_signature(sth, key_override)` | lib.rs:309 | Ed25519 over the canonical STH message |
| `verify_witness_cosign(sth, witness_sigs)` | lib.rs:340 | |
| `verify_bundle_hash(...)` | lib.rs:370 | Bundle integrity |

Wraps `eel-crypto-rs` primitives including `ct_eq_fixed`. Tests: `tests/rfc6962_vectors.rs` (known-answer vectors). FFI bindings feed `packages/kit-sdk-flutter` (`lib/src/verifier_ffi.dart`); build script `build.rs` generates the C header.

**Consumers.** No service crate manifest references it directly; its consumers are the Flutter SDK and external verifier tooling (not verified further).

---

## 2. Python packages

Covered above where they form triplets: [eel-auth-py (1.9)](#19-auth-triplet-eel-auth-rs-eel-auth-py-eel-auth-ex), [eel-events-py (1.3)](#13-events-triplet-eel-events-rs-eel-events-py-eel-events-ex), [eel-eventstore-py (1.5)](#15-event-store-triplet-eel-eventstore-rs-eel-eventstore-py-eel-eventstore-ex), [eel-hlc-py (below)](#21-eel-hlc-py), [eel-pii-py (1.6)](#16-pii-triplet-eel-pii-rs-eel-pii-py-eel-pii-ex), [eel-resilience-py (1.7)](#17-resilience-triplet-eel-resilience-rs-py-ex), [eel-observability-py (1.8)](#18-observability-triplet-eel-observability-rs-py-ex), [eel-secrets-py (1.10)](#110-secrets-triplet-eel-secrets-rs-eel-secrets-py-eel-secrets-ex), [eel-loadshedding-py (1.11)](#111-load-shedding-triplet-eel-loadshedding-rs-py-ex).

There is **no** `eel-crypto-py`: Python services rely on `eel-events-py`'s local RFC 8785 implementation and PyNaCl-style Ed25519 signing rather than a shared crypto crate.

### 2.1 `eel-hlc-py`

[packages/eel-hlc-py/src/eel_hlc/__init__.py](../../../packages/eel-hlc-py/src/eel_hlc/__init__.py): immutable `Hlc` dataclass with full ordering operators (:41-58), `from_str` (:23), `merge` (:61); `HlcClock` (:66) with `now()` (:77) and `receive(remote)` (:91) mirroring the Rust `merge`; `HlcClockProtocol` (:111) for injection. Wire-compatible `"physical_ms.counter.node"` format. Consumers: none found in `services/**/pyproject.toml` (Python services consume `eel-web-py` only — see gap note).

### 2.2 `eel-web-py` — Django shared web layer

**Purpose.** Shared Django app for the two Django consoles (`abis-console`, `declaration-admin`). Provides auth middleware, SpiceDB permission backend, audit + PII-scrubbing middleware, outbox/idempotency/dedupe models, audit-shadow model mixin, and a realm seeding command.

| Item | Location | Description |
|---|---|---|
| `EelAuthMiddleware` | middleware.py:30 | Attaches verified Subject to request |
| `EelAuditMiddleware` | middleware.py:67 | Request audit logging |
| `EelPiiScrubberMiddleware` | middleware.py:84 | Response/log scrubbing |
| `SpiceDBBackend` | backends.py:12 | Django auth backend delegating permission checks to SpiceDB |
| Models: `OutboxRecord`, `IdempotencyKey`, `DeduplicatedEvent`, `AuditShadowBase` / `AuditShadowMixin` | models.py:8/:23/:35/:43/:56 | Mirrors the platform tables in Django ORM |
| `manage.py eel_seed_realms` | management/commands/eel_seed_realms.py:10 | Seeds Keycloak realms |

Migrations: `src/eel_web/migrations/0001_initial.py`. Templates under `src/eel_web/templates/eel_web/`.

**Consumers.** `services/abis-svc/django/abis-console/pyproject.toml` and `services/rts-svc/django/declaration-admin/pyproject.toml` — these are the only Python packages consumed by services.

**Security-relevant behaviors.** Permission decisions centralized in the SpiceDB backend; audit middleware records every mutating request; PII scrubber prevents accidental leakage in console logs.

Tests: `tests/ci/test_web.py`.

---

## 3. Elixir packages

All covered in triplets above: [eel-auth-ex (1.9)](#19-auth-triplet-eel-auth-rs-eel-auth-py-eel-auth-ex), [eel-events-ex (1.3)](#13-events-triplet-eel-events-rs-eel-events-py-eel-events-ex), [eel-eventstore-ex (1.5)](#15-event-store-triplet-eel-eventstore-rs-eel-eventstore-py-eel-eventstore-ex), [eel-outbox-ex (1.4)](#14-outbox-triplet-eel-outbox-rs-eel-outbox-ex), [eel-hlc-ex (below)](#31-eel-hlc-ex), [eel-pii-ex (1.6)](#16-pii-triplet-eel-pii-rs-eel-pii-py-eel-pii-ex), [eel-resilience-ex (1.7)](#17-resilience-triplet-eel-resilience-rs-py-ex), [eel-observability-ex (1.8)](#18-observability-triplet-eel-observability-rs-py-ex), [eel-secrets-ex (1.10)](#110-secrets-triplet-eel-secrets-rs-eel-secrets-py-eel-secrets-ex), [eel-loadshedding-ex (1.11)](#111-load-shedding-triplet-eel-loadshedding-rs-py-ex).

### 3.1 `eel-hlc-ex`

[packages/eel-hlc-ex/lib/eel/hlc.ex](../../../packages/eel-hlc-ex/lib/eel/hlc.ex): pure functions `new/2` (:24), `from_string/1` (:33), `to_string/1` (:55), `merge/2` (:67), `tick/2` (:88), `compare/2` (:98). Stateful wrapper `Eel.Hlc.Clock` GenServer (hlc/clock.ex:1) with `now/1` (:31) and `observe/2` (:37). No mix.exs consumers found in `services/`.

---

## 4. Non-code and multi-language packages

### 4.1 `authz-schema` — SpiceDB schemas

Canonical `.zed` schema per bounded context (24 files: `abis.zed`, `ai_platform.zed`, `boundary.zed`, `bvi.zed`, `bvr.zed`, `crms.zed`, `custody.zed`, `cvr.zed`, `ecc.zed`, `field_mobile.zed`, `gazette.zed`, `governance.zed`, `grievance.zed`, `hello.zed`, `iab.zed`, `inspection.zed`, `inventory.zed`, `party.zed`, `portal.zed`, `queue.zed`, `rts.zed`, `support_admin.zed`, `training.zed`, `transparency.zed`, `vprs.zed`). This is the single source of truth for authorization relationships; every `check()` call in the three language tiers validates against these definitions ([packages/authz-schema/README.md:3-8](../../../packages/authz-schema/README.md)).

Conventions (README.md:31-34): every resource type has a `view` relation for lookup queries; permissions named `<action>_<resource>` (e.g. `submit_form34a`, `read_voter_record`); time-fenced operations use SpiceDB caveats. Example trust structure in [rts.zed](../../../packages/authz-schema/rts.zed): distinct staff/RO/CRO/agent/observer/IAB/chair/workload user types; `resource_form34a` separates `producer`, `adjudicator`, `auditor`, `public_reader` relations with permissions `submit_form`, `adjudicate_form`, `read_form`, `read_audit_trail`, `publish_snapshot`, `read_proof`.

Golden tests: `tests/<context>/<persona>.golden.yaml` (24 persona files) run via `just test-authz`; deployment via `just deploy-authz --env staging|production` (README.md:60-70).

### 4.2 `event-schemas` — JSON Schema for CloudEvents

One directory per bounded context (22 contexts), laid out as `<context>/<aggregate>/<event_name>/v<n>.schema.json`, mapping to CloudEvent type `eel.<context>.<aggregate>_<event_name>.v<n>`. Producers validate before publishing via the `KarapaceClient`s in `eel-events-*`; consumers validate on receipt; evolution is additive-only backward compatible ([packages/event-schemas/README.md](../../../packages/event-schemas/README.md)). Shared primitives in `_shared/`: `hlc.schema.json`, `envelope.schema.json`, `uuid7.schema.json`.

### 4.3 `openapi-contracts` — spec-first APIs (ADR-0011)

OpenAPI 3.1 specs per bounded context (~65 context folders including `rts`, `bvr`, `portal`, `transparency`, `credential`, `governance-*` sub-services, `field-mobile-*`, `support-*`, `training-*`, `ussd`, `ivr`, `whatsapp`, `verifier`, and more). Shared components in `_shared/`: `common.yaml`, `components.yaml`, `headers.yaml`, `problem-types.yaml`, `problem-types-t2.yaml`. Conventions enforced by Spectral (see [tools-reference](../tools-reference.md#spectral)): `x-eel-tier` annotations, RFC 9457 error bodies referencing `problem-types.yaml` URIs only, `Idempotency-Key` on mutating operations, `X-Eel-Signature` on Tier-A operations, JSON+CBOR negotiation on all endpoints. Clients in four languages regenerate from these specs via `just gen-clients`.

### 4.4 `api-clients` — generated clients

Committed generated clients in `dart/`, `py/`, `rs/`, `ts/` — never hand-edited; regenerated by `just gen-clients` from the contracts; `verify.sh` checks freshness ([packages/api-clients/README.md](../../../packages/api-clients/README.md)). Dart clients serve `kit-sdk-flutter`; `rs/` clients serve internal service-to-service calls.

### 4.5 `i18n` — locales

Canonical Swahili (`sw/`) and English (`en/`) locale JSON per surface (`common.json`, `rts.json`, `bvr.json`, `portal.json`, `kit.json`, …). Consumed by the Flutter SDK, Astro portals, and Django admin; duplication into services is prohibited. Tooling: `tools/check_missing.sh` (locale parity) and `tools/extract_keys.sh` ([packages/i18n/README.md](../../../packages/i18n/README.md)).

### 4.6 `design-system` — tokens and components

Design Tokens Community Group-format `tokens.json` plus framework libraries `astro/`, `svelte/`, `django/`. All colour pairs claimed WCAG 2.2 AA compliant (>= 4.5:1 contrast) in light and dark mode ([packages/design-system/README.md](../../../packages/design-system/README.md)); automated verification happens via `tools/wcag-audit`.

### 4.7 Service templates

| Template | Mechanism | Output |
|---|---|---|
| `service-template-rs` | `cargo-generate` ([cargo-generate.toml](../../../packages/service-template-rs/cargo-generate.toml)) | Four standard crates (`<svc>-api/-domain/-projections/-worker`), migrations, Kustomize base, loadtest profile, TRACEABILITY.yaml; ships a feature-gated ADR-0011 conformance harness (`conformance` module: health, problem+json, idempotency, signing tests — README table) |
| `service-template-py` | `cookiecutter` ([cookiecutter.json](../../../packages/service-template-py/cookiecutter.json)) | FastAPI + async SQLAlchemy + Pydantic v2 layout pre-wired with `eel-*-py` middleware; conformance suite under `tests/conformance/` |
| `service-template-ex` | Mix template | Phoenix/LiveView app with Ecto repo, Broadway consumer stub, Oban worker, complete `eel-*-ex` dependency set, conformance module |

### 4.8 `kit-sdk-flutter` — offline kit SDK

Secure offline-capable Flutter SDK for kit devices, backed by `verifier-rs` via Dart FFI ([packages/kit-sdk-flutter/README.md](../../../packages/kit-sdk-flutter/README.md)). Library layout under [lib/src/](../../../packages/kit-sdk-flutter/lib/src/): `secure_store.dart` (Isar + SQLCipher, key derived from device attestation), `offline_queue.dart` (at-least-once queue with idempotency keys), `attestation.dart` (Play Integrity / DeviceCheck; attestation JWT sent as `X-Eel-Attestation`), `hlc.dart` (wire-compatible port of `eel-hlc-rs`), `sync_engine.dart` (HLC-based last-write-wins conflict resolution), `time_fence.dart` (election-day window enforcement offline; clock-manipulation duress detection), `duress.dart`, `http_client.dart` (SPKI pinning), `a11y.dart`, `bundle_security.dart`, `verifier_ffi.dart` (FFI bridge to `verifier-rs`), `telemetry/`, `i18n/`.

### 4.9 `data-dictionary` — column classification

Per-context YAML files (19 files) classifying every Postgres column by ADR-0016 tier, retention period, encryption requirement, and data owner ([packages/data-dictionary/README.md](../../../packages/data-dictionary/README.md)). CI validates that every `eel-pii-*`-typed column has a matching entry; the same file drives the audit-shadow trigger generators in `eel-observability-*`.

---

## Known gaps and inconsistencies

1. **Elixir packages unwired.** None of the eight `services/**/ex/*/mix.exs` files declare dependencies on any `eel-*-ex` package; Elixir services use raw Phoenix/Ecto/Broadway deps only. The `eel-*-ex` libraries compile and test standalone but have no production consumers (verified by grepping all `mix.exs` under `services/`).
2. **`eel-observability-rs`, `eel-secrets-rs` unconsumed.** Zero direct references in `services/*/Cargo.toml` despite being fully implemented; equivalent functionality may be re-implemented locally in services (not verified).
3. **No `eel-outbox-py` / `eel-crypto-py`.** Python outboxing goes through the Django `OutboxRecord` model; Python crypto is ad hoc in `eel-events-py`.
4. **Snapshot compaction missing everywhere.** Documented limitation in all three eventstore READMEs; old events accumulate ahead of snapshots.
5. **`JwksVerifier` disables audience validation** (`validate_aud = false`, packages/eel-auth-rs/src/jwks.rs:107) — intentional per code comment absence; flagged here for security review attention.
6. **Python PII `reveal` allows a `None` sink** (eel-pii-py `__init__.py:77`) unlike the Rust API, weakening audit coverage if misused.
