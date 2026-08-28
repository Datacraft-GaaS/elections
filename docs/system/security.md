# Security Architecture and Safeguards

Reference for the security posture of the eel election platform. Every claim is grounded in
code with file citations; weaknesses and deferrals are listed explicitly in
[Section 10](#10-known-weaknesses-and-deferrals). Ground-truth architecture documents:
`docs/architecture/cross-cutting/trust-model.md`, `docs/architecture/cross-cutting/data-tiers.md`,
and the accepted ADRs (index in `docs/adr/README.md`).

> Note on paths: there is no top-level `deploy/` directory in this repo. Deployment
> manifests live under `platform/` (cluster infrastructure), `edge/` (regional edge
> clusters), `environments/` (per-env kustomize roots), and `services/*/deploy/`
> (per-service kustomize bases plus a `service.yaml` placement descriptor).

---

## 1. Identity and Authentication

### 1.1 SPIFFE / SPIRE workload identity

Every workload receives a SPIFFE SVID from the cluster SPIRE agent
(`docs/architecture/cross-cutting/trust-model.md:5`). mTLS is mandatory for all
inter-service calls; plain HTTP is blocked at the mesh layer (same source).

- Registration entry templates: `platform/spire/registration-entries.yaml` defines one
  SPIFFE ID per service account, e.g.
  `spiffe://eel.prod/ns/rts-svc/sa/rts-api` with `k8s_psat` node selectors and TTL 3600
  (abis-svc gets TTL 1800). Applied via `kubectl apply -f platform/spire/registration-entries.yaml -n spire`.
- Per-service entry files: `platform/spire/entries/<svc>-svc.yaml` for all 18 services,
  plus `federation-dr-primary.yaml`, `federation-dr-mirror.yaml` (cross-site federation)
  and `trust-bundle-exchange.yaml`.
- Kyverno policy `platform/kyverno-policies/require-svid-label.yaml` enforces SVID
  labelling on workloads.

### 1.2 Keycloak / OIDC (user-facing tokens)

Keycloak is the user authentication provider (ADR-0007). Realm configs live under
`platform/keycloak/realms/`: `staff.json`, `observer.json`, `party.json`, `media.json`,
`county.json`, `public.json`, `eel-realm.json`; policies include WebAuthn
(`platform/keycloak/config/webauthn-policy.json`) and an events listener
(`events-listener.yaml`).

JWT verification in Rust is provided by the cached JWKS verifier
`JwksVerifier` in `packages/eel-auth-rs/src/jwks.rs`:

- Key cache: moka cache, capacity 64, TTL 300 s (`jwks.rs:53-56`); kid-miss triggers a
  JWKS refetch (`get_key`, `jwks.rs:81-91`).
- Algorithm pinned to RS256 with issuer derived from the JWKS URL
  (`verify`, `jwks.rs:100-104`). RSA and EC keys are both supported
  (`build_decoding_key`, `jwks.rs:129-145`).
- **Audience validation is disabled**: `validation.validate_aud = false` at
  `packages/eel-auth-rs/src/jwks.rs:105`. See Section 10.
- Claims mapped into a `Subject { sub, realm, session_id, roles, purpose_scopes }`
  (`jwks.rs:113-125`).

The shared axum middleware `require_jwt` in
`packages/eel-api-server-rs/src/middleware/auth.rs:14` wraps `JwksVerifier::verify`,
injects the `Subject` into request extensions, and returns an RFC 9457 401 problem
document on failure.

### 1.3 Bearer checks at services

Services behind the gateway perform presence-level bearer enforcement:

- `require_bearer(headers, instance)` in
  `services/bvi-svc/crates/bvi-api/src/idem.rs:81-95` requires a non-empty
  `Authorization: Bearer ...` header; actual token validation happens at the Envoy
  gateway (see the comment at `idem.rs:79-80`).
- Full JWT validation middleware: `eel-api-server-rs/src/middleware/auth.rs` (above).
- ABIS API has its own SPIFFE/SVID middleware — but it is a stub and is **not wired
  into the router**; see Section 10.

### 1.4 mTLS posture per tier

| Tier | Identity | Transport |
|---|---|---|
| Service-to-service (internal mesh) | SPIFFE SVID per workload | mandatory mTLS (`trust-model.md:5`) |
| Kit-to-edge | Kit device certificate from inventory HSM partition + kit API key | HTTPS/mTLS over cellular (`deployment-topology.md:59`) |
| Public API tier | none / OIDC where authenticated | TLS terminated at Envoy Gateway |
| External gov integrations (IPRS/KRA/EACC/eCourt) | client certificate or service-account token | dedicated adapters in `integrations/` (`trust-model.md:75`) |

Gateway classes distinguish exposure: `eel-external` (internet-facing), `eel-internal`
(cluster API routes), `eel-admin` (restricted source CIDRs)
(`platform/envoy-gateway/README.md`).

---

## 2. Authorization

### 2.1 SpiceDB schema

Coarse-grained authorization is enforced by SpiceDB using the schemas in
`packages/authz-schema/*.zed`; "every write path checks a SpiceDB permission before
acting" (`trust-model.md:9`). Tooling lives in `tools/spicedb` and `tools/iam-seed`.

Representative relations and permissions (all files declare the common actor set
`user_staff/user_ro/user_cro/user_agent/user_observer/user_iab/user_chair/service_workload`):

| Resource | File | Key permissions |
|---|---|---|
| Station (BVI) | `packages/authz-schema/bvi.zed:13-31` | `open_session`, `close_session`, `ingest_identification` (owner only), `read_turnout`, `revoke_kit` (chair/cro/ro only), `send_distress` |
| Voter register | `packages/authz-schema/bvr.zed:13-29` | `write_voter` (owner only), `freeze_register`, `export_register`, `process_sar` (DPA subject access), `approve_bulk_export` (dual approver), `resolve_adjudication` |
| Form 34A | `packages/authz-schema/rts.zed:13-24` | `submit_form` (producer), `adjudicate_form`, `read_form` (incl. public_reader), `read_audit_trail`, `read_proof` |
| Transparency leaf | `packages/authz-schema/transparency.zed:13-22` | `append_leaf` (log_writer), `read_sth`, `verify_inclusion`, `verify_consistency` (public_reader), `cosign_sth` (witness) |
| ABIS match | `packages/authz-schema/abis.zed` | `submit_match` (workload), `resolve_adjudication` (adjudicator), `trigger_batch_dedup`, `read_reports` |

Dual-control patterns are explicit in schema: bulk register export needs
`dual_approver` (`bvr.zed:27`); kit revocation excludes station officers entirely
(`bvi.zed:23`).

### 2.2 OPA / Kyverno policy locations

- OPA bundles: `platform/opa-bundles/admission/` and `platform/opa-bundles/authz/`
  (`platform/opa-bundles/README.md`).
- Kyverno cluster policies in `platform/kyverno-policies/`: `forbid-privileged.yaml`,
  `forbid-hostpath.yaml`, `forbid-latest-tag.yaml`, `forbid-nodeport.yaml`,
  `require-probes-and-limits.yaml`, `require-signed-images.yaml`,
  `require-otel-annotations.yaml`, `require-svid-label.yaml`.

### 2.3 `x-eel-tier` conventions in openapi-contracts

Operations in `packages/openapi-contracts/<ctx>/openapi.yaml` carry an `x-eel-tier`
extension used for routing/authz posture, e.g. bvi operations tagged
`x-eel-tier: internal` (station/kit endpoints,
`packages/openapi-contracts/bvi/openapi.yaml:83,149,212,290,336,395`) and public tiers
`x-eel-tier: 3` (line 460) and `x-eel-tier: 2` (line 547).

Discrepancy note: `packages/openapi-contracts/README.md` ("Contract conventions")
documents tiers as `x-eel-tier: A|B|C matching data-class`, but the actual contract
values use `internal` / numeric tier labels. The Spectral ruleset
(`packages/openapi-contracts/.spectral.yaml`) is the enforcement point; see
Section 10 for the doc-vs-code mismatch.

---

## 3. Cryptography

### 3.1 eel-crypto-rs primitives

Package: `packages/eel-crypto-rs/` (modules declared in `src/lib.rs:21-38`).

Merkle tree — RFC 6962 Certificate Transparency semantics over SHA-256
(`src/merkle.rs:1-8`):

| Function | Location | Purpose |
|---|---|---|
| `leaf_hash` | `merkle.rs:40` | `SHA256(0x00 ‖ data)` |
| `node_hash` | `merkle.rs:58` | `SHA256(0x01 ‖ left ‖ right)` |
| `MerkleTree::build` / `from_leaf_hashes` | `merkle.rs:101` / `:128` | tree construction |
| `root` | `merkle.rs:149` | current root hash |
| `inclusion_proof` / `verify_inclusion` | `merkle.rs:175` / `:200` | inclusion proofs (also root-checked variants at `:228`, `:258`) |
| `consistency_proof` / `verify_consistency` | `merkle.rs:301` / `:320` | proof that an earlier tree is a prefix of the current tree |

JSON canonicalization — RFC 8785 JCS: `canonicalize(&Value) -> Vec<u8>` at
`packages/eel-crypto-rs/src/rfc8785.rs:44`; canonical bytes are the signing input for
signatures so verifiers reproduce them byte-for-byte.

Signing:

- Ed25519: `generate_keypair` (`ed25519.rs:29`), `sign` (`ed25519.rs:47`),
  `verify` (`ed25519.rs:65`), key/signature deserialization guards
  (`ed25519.rs:86,102`).
- ECDSA P-256 verification inside the cosign bundle verifier
  (`cosign.rs:80-90`, accepts DER or fixed-width signatures).
- Post-quantum hybrid (Ed25519 + ML-DSA style dual signature) and ML-KEM:
  `HybridKeyPair::{generate,sign,verify}` (`pq.rs:62,77,99`), `generate_keypair` for
  KEM (`pq.rs:144`). Supports ADR-0005's TLS-1.3 + PQ-hybrid direction.
- Cosigned artifact bundles: `CosignBundle` + `verify_bundle` (`cosign.rs:40,54`).
- Timestamping client (RFC 3161 TSA): `TsaClient` (`tsa.rs:52,59`).
- BLAKE3 hashing: `blake3.rs` (used by the M14 tally replay).

Constant-time comparison — uses the `subtle` crate; direct `==` on secret byte slices
is disallowed by the Semgrep rule `semgrep-rules/no-eq-on-secrets.yml`
(`constant_time.rs:1-4`):

- `ct_eq(a, b)` (`constant_time.rs:32-40`) — length mismatch leaks no positional
  information; compares a dummy equal-length pair before returning false.
- `ct_eq_fixed::<N>(a, b)` (`constant_time.rs:51`).
- Property-tested against `==` equivalence (`constant_time.rs:80-87`).

PKCS#11 session pooling for HSM access: `PkcsPool::open/get` and role-based key
lookup `find_key_by_role` (`pkcs11_pool.rs:82,126,188`).

### 3.2 HSM integration

ADR-0005 (`docs/adr/0005-crypto-hsm.md`) mandates PKCS#11/KMIP with M-of-N key
ceremony. Implementation surfaces:

- `tools/hsm-init/` — bootstraps PKCS#11 keys in SoftHSM slots mirroring the production
  slot layout (`tools/hsm-init/README.md`); loads the module via cryptoki
  (`Pkcs11::new` at `tools/hsm-init/src/main.rs:152`) with subcommands `init-slots`,
  `list`, `destroy`, `verify`, plus sign/verify round-trip (`main.rs:176,183`).
- `tools/hsm-ceremony/` — custodian CLI for M-of-N ceremonies: custodian registration
  with Ed25519 raw pubkeys, `ceremony request --quorum-m --quorum-n`, per-custodian
  `ceremony approve` producing a detached signature over the operation hash, `status`,
  and abort while non-terminal (`tools/hsm-ceremony/README.md`).
- Server-side ceremony state machine: `services/transparency-svc/crates/hsm-signer/`
  — `CeremonyState` (`ceremony.rs:41`), `Ceremony { quorum_m, quorum_n }`
  (`ceremony.rs:69-74`), approval counting (`approval_count`, `ceremony.rs:106`);
  request defaults to quorum 3-of-5 (`lib.rs:241-242`).
- HSM roles per trust model: RTS partition signs Form 34* and Merkle roots; BVR/Training
  partition issues kit/worker certificates; ABIS session keys derived per session via
  PKCS#11 (`trust-model.md:63-67`).
- Local dev: `tools/softhsm/init.sh`.

### 3.3 Transparency log signing chain

ADR-0017 (`docs/adr/0017-transparency-log.md`): single monolithic append-only Merkle
log per election; STH cadence hot=1s / default=60s / batch=5min; STHs HSM-signed;
leaves producer-signed Ed25519 and verified before hashing.

Implementation in `services/transparency-svc/`:

- Cadence producers: `SthProducer::spawn_all` spawns Hot/Default/Batch tasks
  (`crates/tlog-sequencer/src/sth_producer.rs:22-29`); each computes tree size from
  the last leaf and the full-tree root via `proofs::current_root`.
- **STH signatures are real**: every head is signed before persistence over
  `tree_size || root_hash || cadence` by the configured backend — PKCS#11
  `eel-sth` key (CKM_ECDSA P-384; softHSM in dev, network HSM in production) or,
  only when explicitly selected with `EEL_STH_SIGNER=dev`, a local Ed25519 key
  (`crates/tlog-sequencer/src/signer.rs`). Signature and key fingerprint are stored
  on the row; startup aborts if no signer can be configured.
- Witness cosigning permission exists in SpiceDB (`transparency.zed:22`,
  `cosign_sth = witness`) consistent with ADR-0017 split-view prevention.

---

## 4. Data Protection

### 4.1 PII package semantics

Three language implementations of the ADR-0016 five-tier classification
(A restricted / B sensitive / C operational-D public / E ephemeral):

Rust — `packages/eel-pii-rs/src/lib.rs`:

- Wrapper table documented at `lib.rs:12-18`: `PIIRestricted<T>` (tier A, column-level
  encryption required), `PIISensitive<T>` (B, TDE), `PIIPublic<T>` (C),
  `PIIOperational<T>` (D), `PIIEphemeral<T>` (E, TTL-managed).
- Access to A/B values requires an explicit `Purpose` (enum at `lib.rs:44`, variants:
  Enrolment, Identification, Adjudication, Publication, StatutoryReport,
  GrievanceIntake, SupportTicket, Audit, DataPortability, ErasureRequest,
  SecurityInvestigation, InternalMonitoring) **and** a `PIIAuditSink` trait object
  recording `(data_class, purpose, type_name)` (`lib.rs:110+`).
- Redaction by construction: `Debug` and `Serialize` impls on `PIIRestricted` /
  `PIISensitive` never emit the value — serialization errors with a message directing
  callers to `.as_redacted()` or `.reveal(purpose, sink)`
  (`lib.rs:237-255`, `:336-352`). This is the dump-forbidden protection: accidental
  `serde_json::to_string(&record)` or log formatting cannot leak tier-A/B fields.
- Reveal is audited: `reveal(purpose, sink)` at `lib.rs:214` (PIIRestricted) and
  `lib.rs:313` (PIISensitive).

Elixir — `packages/eel-pii-ex/lib/eel/pii/ecto_types.ex`: Ecto type `dump/1` for
restricted values returns `{:error, :pii_dump_forbidden}` (`ecto_types.ex:28`),
blocking persistence through unannotated columns; sensitive dumps require explicit
value extraction (`ecto_types.ex:58`).

Python — `packages/eel-pii-py/src/eel_pii/__init__.py`: pydantic-native wrappers +
SQLAlchemy `TypeDecorator` column types (`pii_column_type`,
`PIIRestrictedType`, ...); `Purpose` and `PIITier` enums mirror the Rust semantics;
async `PIIAuditSink` protocol (`__init__.py:47-50`).

### 4.2 Data tiers

Two related models coexist:

- Storage topology tiers T0-T3 in `docs/architecture/cross-cutting/data-tiers.md:37-45`:
  T0 biometric (enclave only), T1 personal NID/name/DoB (10-year retention, Elections
  Act s.44), T2 operational (7 years), T3 aggregate/public record.
- Field-classification tiers A-E of ADR-0016 enforced by the `eel-pii-*` libraries
  (above).

T0/T1 data sovereignty: Kenyan-hosted infrastructure only; diaspora batches encrypted
in transit and at rest with HSM-derived keys (`data-tiers.md:62-64`).

Schema evolution is additive-only expand-migrate-contract (ADR-0014 summary at
`data-tiers.md:58-60`); Kafka schemas versioned in `packages/event-schemas/` behind
the Karapace registry.

### 4.3 Encryption at rest / in transit references

- Transit: TLS 1.3 + PQ hybrid direction (ADR-0005), mTLS mesh (Section 1.4),
  kit-to-edge HTTPS/mTLS (`deployment-topology.md:59`).
- At rest: tier rules in `trust-model.md:52-61` (T0 enclave/HSM-encrypted only; T1
  encrypted at rest with audited access; T2 standard encryption); column-level
  encryption requirement encoded as tier-A wrapper semantics (4.1).
- Secrets/keys: OpenBao mounts incl. `transit` (envelope encryption service) and `pki`
  (`platform/openbao/mounts/{transit,pki}.yaml`); see also Section 7 of the operations
  reference.

### 4.4 Audit tables

Voter lookup auditing (FR-M07-020/021): every BVV register lookup writes a
non-erasable row to `voter_lookup_audit` in the same transaction as the read —
hashes of actor session, target voter id, and IP, plus result status
(`services/bvi-svc/crates/bvi-api/src/store.rs:586-596`). The store refuses to invent
station names: descriptive metadata must exist in the `polling_stations` mirror or the
lookup reports not-found (`store.rs:563-565`).

Event-level audit shadowing helper: `audit_shadow_sql(table_name)` in
`packages/eel-observability-rs/src/audit_shadow.rs:8` (stateful services keep an audit
shadow per ADR-0013, cf. `data-tiers.md:46-56`).

---

## 5. Edge and Availability Safeguards

### 5.1 Edge stack

Five regional k8s clusters (Nairobi, Mombasa, Kisumu, Eldoret, Garissa) with anycast
VIPs and BGP peering absorb public traffic before it reaches the core
(`edge/README.md`; region table `edge/README.md:8-14`; failover by BGP withdrawal in
60-180 s, `edge/README.md:33-36`). Each regional overlay ships kustomize manifests:
`kustomization.yaml`, `dns-vip-configmap.yaml`, `envoy-gateway-patch.yaml`,
`network-policy.yaml`, `node-selector-patch.yaml`, `argocd-appset.yaml` (e.g.
`edge/nairobi/`).

Layer roles (documented in `edge/README.md:25-31`):

| Component | Directory | Role |
|---|---|---|
| Envoy Gateway | `platform/envoy-gateway/`, per-region patches | north-south ingress, gateway classes `eel-external/internal/admin`, sliding-window rate limits |
| nginx origin shield | `edge/nginx/` | upstream routing, TLS termination config |
| Varnish | `edge/varnish/` | long-TTL caching of signed static snapshots; live results cached 30 s stale-while-revalidate; caches the `X-Eel-Snapshot-Sig` signature alongside bodies (`edge/varnish/README.md`) |
| Coraza WAF | `edge/waf-rules/` | hand-tuned per-service rules instead of default CRS; SQLi on voter lookup, XSS on results, replay detection on RTS, kit UA allowlists, CBOR content-type exclusions (`edge/waf-rules/README.md`) |
| Anubis PoW | `edge/anubis/` | three-step escalation PASS -> CHALLENGE (SHA-256 partial preimage, solved token cached in Valkey 10 min) -> BLOCK; signed-JWT bypass token for kit firmware (`edge/anubis/README.md`) |

Status caveat: these four shared-config directories currently contain only their
README plus `.gitkeep`; the concrete rule/config files they document are not yet in
the repository. See Section 10.

### 5.2 Rate limiting

Edge tier table (`edge/rate-limits/README.md`): global IP 300 req/min; portal 120;
results 60 req/min; kit API 200 req/min keyed by kit API key; admin 30 req/min per
bearer; voter lookup 10 req/min per IP+endpoint. Counters stored in Valkey; excess
requests get `429` + computed `Retry-After`; election-day burst overrides applied at
T-30 min.

Service-side backpressure: `SequencerRateLimiter` bounds concurrent tlog appends with
a tokio semaphore; producers that cannot acquire a permit get 429 + Retry-After
(`services/transparency-svc/crates/tlog-sequencer/src/rate.rs:6-24`).

### 5.3 Load shedding and circuit breakers

Shared libraries make resilience "by construction" (ADR-0009,
`docs/adr/0009-resilience-posture.md`):

- Load shedding: `eel-loadshedding-*`. CoDel-inspired shedder with bounded queue
  (`CoDelShedder`, `packages/eel-loadshedding-rs/src/shedder.rs:45`); request priority
  classification from headers (admin > user > accredited machine > anonymous;
  `Priority::from_headers`, `priority.rs:27`); shedding returns `503 Retry-After`
  problem+json per ADR-0009.
- Circuit breakers: `CircuitBreaker` with closed/open/half-open states
  (`packages/eel-resilience-rs/src/circuit_breaker.rs:7,29,41`).
- Degradation modes: `DegradationMode` flag (normal/degraded/read-only-lockdown;
  `degradation.rs:8,25`) enforced at the framework level by the write-guard middleware
  `guard_writes` (`packages/eel-api-server-rs/src/middleware/degradation_guard.rs:6`).
- Bulkheads, retry budgets, timeouts: sibling modules `bulkhead.rs`, `retry.rs`,
  `timeout.rs` in the same package. Elixir/Python equivalents:
  `packages/eel-loadshedding-{ex,py}`, `packages/eel-resilience-{ex,py}`.

---

## 6. Integrity and Provenance

### 6.1 Transactional outbox and event signing

`packages/eel-outbox-rs/` implements the ADR-0015 outbox: `Outbox::emit` inserts a
CloudEvents envelope row inside the caller's transaction; Debezium publishes it, and
rollback discards it (`outbox.rs:44-66`). Guards: payload cap of 1 MiB
(`PayloadTooLarge`, `outbox.rs:55-59`), UUID validation.

Envelope integrity headers (`envelope.rs`, serialized into Kafka headers at
`outbox.rs:70-88`): `eel_prev_hash` (hash chain per stream), `eel_seq`, `eel_hlc`
(hybrid logical clock), `eel_producer_key_id` (SPIFFE SVID fingerprint of the signing
key, `envelope.rs:39`), and optional `eel_signature` — base64 Ed25519 signature over
the envelope (`envelope.rs:41-42`, decoded accessor `signature_bytes` at `:69`).
Consumers may treat unsigned topics explicitly (`outbox.rs:91`). Event-sourcing scope
and signing requirements: ADR-0013/0015; topic conventions:
`docs/architecture/cross-cutting/event-flow.md`.

### 6.2 Idempotency (ADR-0011 implementations)

Mandatory `Idempotency-Key` on every write (ADR-0009/0011). Reference plumbing in
`services/bvi-svc/crates/bvi-api/src/idem.rs`:

- `require_key` validates a UUID key (`idem.rs:19-26`).
- `replay` returns the snapshotted success response (always status 200) without
  re-execution; error outcomes are never snapshotted (`idem.rs:31-48`).
- `snapshot` persists success responses (`idem.rs:51-65`); backing storage includes
  lazy expiry sweep on `request_fingerprints.expires_at` (`store.rs:611-616`).
- Shared framework middleware: `idempotency_check`
  (`packages/eel-api-server-rs/src/middleware/idempotency.rs:8`); ABIS variant
  `require_idempotency_key` gates mutating methods
  (`services/abis-svc/crates/abis-api/src/middleware.rs:53-78`).

### 6.3 Kit attestation

`services/biometric-edge-svc/crates/kit-attest/` verifies device attestation payloads
with kinds PlayIntegrity / AppAttest / TPM quote / legacy software (`verify.rs:10-15`)
and produces an `AttestationVerdict` with device tier (`KitTier`: Tpm /
PlayIntegrity / AppAttest / Legacy, `verdict.rs:7-18`), integrity level
(MeetsStrong/MeetsDevice/MeetsBasic, `verdict.rs:21-30`), expiry, cert thumbprint and
a legacy warning flag (`verdict.rs:32-46`). Note: `verify_attestation`
(`verify.rs:50`) is a well-structured stub — production calls to Google/Apple/TPM
validation chains are pending (comment at `verify.rs:47-49`). See Section 10.

Device certificates are revoked within 60 s when a kit is decommissioned or reported
lost (`trust-model.md:71`); the revocation endpoint is
`POST /api/v1/bvi/kits/{serial}/revoke` (`bvi.zed` `revoke_kit`;
`services/bvi-svc/crates/bvi-api/src/routes/kits.rs`).

### 6.4 Kit certificate transparency

`services/biometric-edge-svc/crates/kit-cert-transparency/` runs a CT-style log of kit
certificates: append-only log with `SignedTreeHead` and `InclusionProof`
(`log.rs:33-46`), append + STN maintenance (`append`, `current_sth`,
`log.rs:68,86`), inclusion proofs (`inclusion_proof`, `log.rs:106`), issuance hook
that intercepts certificate issuance and returns receipts (`issuance_hook.rs:51`),
and REST endpoints for append/proof (`api.rs:126`). Leaf serialization is deterministic
(`entry.rs:40`).

### 6.5 Reproducibility as provenance

Every build artifact can be rebuilt from source and compared byte-for-byte; tally can
be replayed to re-derive the BLAKE3 Merkle root and compared against the signed hash
manifest anchored in the transparency log (`reproducibility/README.md`). Pipeline
detail is in the operations reference, section 1.

---

## 7. Enumeration and Timing Defences

BVV voter lookup (`services/bvi-svc/ex/bvv_web/lib/bvv_web/live/lookup_live.ex`):

- Minimum 300 ms response plus random jitter: `@min_response_ms 300` (line 16);
  elapsed-time compensation and `:rand.uniform(50)` jitter with `Process.sleep`
  (`lookup_live.ex:30-36`). Tagged EC-SEC-10 in the module docs (line 5).
- Identical payload shapes regardless of existence: hits return the populated map
  (lines 46-55); misses, upstream failures, and exceptions all collapse into the same
  nil-field shape via `not_found_result/0` (lines 57-69, `71-81`), so a 404 is
  indistinguishable in shape from an error path.
- Every lookup is server-side audit-logged whether found or not (FR-M07-020), with
  non-erasable rows (FR-M07-021) — see 4.4.
- Edge rate limit specifically for lookup: 10 req/min per IP+endpoint
  (`edge/rate-limits/README.md`, "Voter lookup" tier); WAF enumeration guard noted for
  BVR search (`edge/waf-rules/README.md`, "enumeration rate guard").
- Constant-time secret/hash comparison: `ct_eq` / `ct_eq_fixed`
  (`packages/eel-crypto-rs/src/constant_time.rs:32,51`) mandated by Semgrep rule
  `semgrep-rules/no-eq-on-secrets.yml`.
- Error responses never leak internals: RFC 9457 problem+json everywhere (ADR-0011;
  shared problem types in `packages/openapi-contracts/_shared/problem-types.yaml`).

---

## 8. Compliance Hooks

| Hook | Location | What it does |
|---|---|---|
| Trace markers | `packages/eel-trace-rs/src/lib.rs:23` | No-op `#[trace("EC-DAT-01", ...)]` proc-macro carrying requirement IDs on tests so the completeness auditor can map tests to requirements without altering behavior |
| Completeness scanner | `tools/completeness/` | Stub/skip/todo scan (`--strict`, `--fail-on-new`, allowlist `.completeness-allowlist.yaml`), traceability coverage from test tags, event contract closure (`tools/completeness/README.md`) |
| ASVS check | `tools/asvs-check/` | Maps repo trace tags to OWASP ASVS 4.0 L2/L3 requirements via `asvs-manifest.yaml` (`mapped_tags` per requirement); recognizes Rust/Python/Elixir/Dart/JS tag syntaxes (`tools/asvs-check/README.md`) |
| WCAG audit | `tools/wcag-audit/` | axe-core audits against a URL manifest; AA for most surfaces, AAA for M68 referendum-explainer; CI stub mode via `EEL_WCAG_STUB_DIR` (`tools/wcag-audit/README.md`) |
| PII-in-events checker | `tools/pii-in-events/` | HARD-24: Semgrep-rule linter failing PRs that put Tier-A/B PII into event schemas (`tools/pii-in-events/pyproject.toml` description); wired as Woodpecker gate `.woodpecker/pii-lint.yaml` |
| Evidence collection | `verification/security/` | `asvs-evidence-collector.sh`, `sast-sca-container.sh`, DAST (ZAP baseline + nuclei templates), fuzz corpus runner, pentest scope tracker |

---

## 9. Threat Model Summary

Mapping of major threats to concrete controls (all cited above):

| Threat | Controls |
|---|---|
| Voter enumeration / timing oracle | 300 ms + jitter identical-shape lookups (`lookup_live.ex:16,30-36,71-81`); per-endpoint rate limit 10 req/min (`edge/rate-limits/README.md`); WAF enumeration guard (`edge/waf-rules/README.md`); hashed, non-erasable audit rows (`store.rs:586-596`); PII wrappers prevent bulk export leakage (`eel-pii-rs` Serialize redaction, `lib.rs:251-255`) |
| Ballot stuffing / fake Form 34 | SpiceDB `submit_form` restricted to producer role (`rts.zed:19`); forms signed, ICR validation + Merkle anchor on RTS (`event-flow.md:26-27`); every consensus-relevant event signed and Merkle-anchored (ADR-0011; outbox `eel_signature`/`eel_prev_hash`, `envelope.rs:39-42`); transparency log gives inclusion proofs (`merkle.rs:175-228`); third-party reproduce-challenge replays tallies independently (`verification/reproduce-challenge/README.md`) |
| Result tampering after publication | Append-only log + consistency proofs (`merkle.rs:301,320`); STH cross-posting to witnesses/Gazette prevents split-view (ADR-0017; witness `cosign_sth` permission `transparency.zed:22`); Varnish caches snapshot signature `X-Eel-Snapshot-Sig` alongside bodies (`edge/varnish/README.md`); cosign bundle verification (`cosign.rs:54`) |
| Insider threat / privilege abuse | Dual-control exports (`approve_bulk_export` dual_approver, `bvr.zed:27`); kit revocation withheld from station staff (`bvi.zed:23`); M-of-N HSM ceremonies with independent custodian approvals (`tools/hsm-ceremony/README.md`; `ceremony.rs:69-74`); JIT admin credentials capped at 4 h (`packages/eel-secrets-rs/src/lib.rs:5-8`); purpose-bound audited PII reveal (4.1); IAB observer roles across schemas |
| Kit theft / tampering | Device attestation verdicts with hardware tiers (`kit-attest/verdict.rs`); kit cert transparency log with issuance hook and proofs (6.4); 60 s revocation on loss (`trust-model.md:71`); offline kits degrade gracefully (ADR-0009 modes) |
| DoS / request floods | Edge rate-limit tiers + burst overrides (`edge/rate-limits/README.md`); Anubis PoW escalation (`edge/anubis/README.md`); Coraza WAF (`edge/waf-rules/README.md`); CoDel load shedding with priority classes (`shedder.rs:45`, `priority.rs:27`); circuit breakers (`circuit_breaker.rs:29`); sequencer semaphore backpressure (`rate.rs:8-24`); anycast/BGP regional absorption (`edge/README.md:24-36`) |
| Supply-chain / wrong-binary attacks | Nix hermetic rebuild + diffoscope zero-diff + tally replay + manifest signature check (`reproducibility/README.md`); cosign image-signing Kyverno policy (`platform/kyverno-policies/require-signed-images.yaml`); SLSA L3 posture (ADR-0006) |
| Split-view / log rewrite | Witness cosigning + Gazette anchoring (ADR-0017); consistency-proof verification API; single canonical log contract |

---

## 10. Known Weaknesses and Deferrals

Verified in code as of this writing. Do not remove entries without fixing the code.

1. **OIDC audience validation disabled.** `validation.validate_aud = false` in
   `packages/eel-auth-rs/src/jwks.rs:105`. Tokens from the realm are accepted for any
   audience; mitigation would pin the expected `azp`/`aud` per service.
   (Note: sometimes referenced as line 107; the assignment is at line 105.)
2. RESOLVED (was: placeholder STH signatures). `crates/tlog-sequencer/src/signer.rs`
   now signs every STH via PKCS#11/softHSM (`eel-sth`, ECDSA P-384) or an explicit dev
   Ed25519 key. Residual risk moves to deployment: production trust requires exercising
   the network-HSM path and the M-of-N custodian ceremony (`crates/hsm-signer`).
3. **ABIS auth middleware unwired and stubbed.** `spire_auth`
   (`services/abis-svc/crates/abis-api/src/middleware.rs:15-47`) defaults to dev-bypass
   unless `EEL_ABIS_SPIRE_REQUIRED=true`, and even then only rejects the literal token
   `"invalid"` rather than validating a SPIFFE SVID (comment at `middleware.rs:31-33`).
   The router never registers it anyway — `app()` layers only `cosign_response`
   (`abis-api/src/lib.rs:103`).
4. **ABIS response signing is a placeholder.** `cosign_response`
   (`middleware.rs:84-92`) stamps a constant `x-eel-signature: stub-cosign-v1` header;
   no actual signature is computed.
5. **Hardcoded WhatsApp webhook verify token.**
   `services/field-mobile-svc/py/wa-bot/src/wa_bot/main.py:26` compares
   `hub_verify_token` against the literal `"eel-verify-token"`; must move to a secret
   (OpenBao) before production. The POST webhook handler additionally performs no
   Meta signature validation (`main.py:31-34`).
6. **Kit attestation verification is a structured stub.** `verify_attestation`
   (`services/biometric-edge-svc/crates/kit-attest/src/verify.rs:47-50`) documents
   itself as such; Play Integrity / App Attest / TPM validation chains are not called.
7. **Edge shared configs are documentation-only.** `edge/nginx/`, `edge/varnish/`,
   `edge/anubis/`, `edge/rate-limits/`, `edge/waf-rules/` contain README.md and
   `.gitkeep` only; the rule/config files their READMEs tabulate (e.g.
   `limits.yaml`, `rules/bvr.conf`, Anubis `config.yaml`) do not exist in-repo yet.
8. **Contract tier annotation convention drift.**
   `packages/openapi-contracts/README.md` specifies `x-eel-tier: A|B|C`, while shipped
   contracts use `internal` / `2` / `3` (e.g. `bvi/openapi.yaml:83,460,547`).
