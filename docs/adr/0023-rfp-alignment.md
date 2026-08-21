# ADR 0023: RFP-alignment reconciliation

Status: accepted
Date: 2026-08-21
Supersedes-in-part: 0004, 0005, 0006, 0007, 0008, 0012, 0015, 0016, 0017, 0018

## Context

After ADRs 0001-0018 were committed, the IEBC IEMS spec corpus (`work_docs/01-Projects/IEBC/sw/`) was substantially expanded and a review pass was run against both the updated corpus and the RFP source (`work_docs/01-Projects/IEBC/tender.txt` + `PRE_BID_CLARIFICATIONS.md`). The review revealed:

1. **The RFP mandates almost nothing at framework level.** Where the spec doc pins Next.js / Cedar / Vault / Linkerd / Kotlin / Avro, those are Datacraft-drafted, not RFP-derived. Per user policy: "unless derived from the actual RFP, we can make the choice that is optimal."
2. **The RFP mandates several concrete constraints we had not fully captured**, particularly around DR site count, log retention window, concurrent-session SLA, and specific integrations.
3. **Several new architectural elements** (5 regional edge clusters, 18-service K8s topology grouping 69 modules, confidential compute for ABIS, per-cycle KEK hierarchy, mobile device attestation) are Datacraft-drafted extensions we accept as sound.

This ADR records the amendments the review triggered. The original ADRs remain the canonical decision text for their subject; this ADR is the single audit-trail document for the RFP-alignment pass.

## Decision (amendments)

### ADR-0004 (Platform): amendments

- **Three-site topology → two IEBC-provided DR sites** (primary + mirror). RFP mandates two (tender.txt Sec V; PRE_BID C1). A third defence-in-depth site is optional (reproducibility-bundle mirror only) and budget-dependent.
- **Add 5-zone network segmentation** (Edge / DMZ / Core / Air-gapped / Management) with hardware firewall + K8s NetPol per `sw/SECURITY.md` §2.3.
- **Reference ADR-0019** for the 5 regional edge Kubernetes clusters.
- **Reference ADR-0020** for the 18-service K8s topology grouping of the 69 modules.
- Node OS is Ubuntu 24.04 LTS (earlier draft said 22.04; CIS L2 benchmark exists for 24.04).

### ADR-0005 (Crypto & HSM): amendments

- **FIPS level closed to 140-3 L3+.** `expect.md`'s 140-2 L3 is stale; ARCHITECTURE + SECURITY concur on 140-3 L3+. Remove the deferred placeholder.
- **Extend PQ suite**: add **ML-DSA + ECDSA P-384 hybrid signatures** and **SLH-DSA (FIPS 205)** for archive re-signing (long-tail 15-year retention). ML-KEM-768 + X25519 for KEM (already committed).
- **Confidential compute for ABIS**: **AMD SEV-SNP or Intel TDX** enclaves for `abis-svc` (see ADR-0020) with **remote attestation**. Adds a hardware-selector requirement to the ABIS nodes in the K8s cluster manifest.
- **Per-election-cycle KEK hierarchy**, every election cycle has its own KEK, sealed by the HSM. Cycle KEKs escrow to OAG post-election. **See ADR-0016 amendment** for how this pairs with per-record erasure.

### ADR-0006 (Build & supply chain): amendments

- **Add Pact** for cross-service contract testing (every service-pair contract).
- **Add CycloneDX** as an SBOM output format alongside SPDX (both generated per artifact).
- **Add Semgrep + Bandit + SpotBugs** to the SAST toolchain (in addition to cargo-clippy / cargo-audit / pip-audit / pnpm audit).
- **Add RFC 3161 timestamps** for evidentiary chain on release artifacts.
- **Add BLAKE3** as the high-throughput local hash for kit audit logs (SHA-256 remains for cross-system signatures).

### ADR-0007 (AuthN + AuthZ + Secrets): amendments

- **Add JIT dynamic secrets**: OpenBao issues short-lived (max 4h) database and admin credentials per session.
- **Add maximum admin session length of 4 hours** with mandatory session recording (SSH via `sshaudit` + Web via BFF audit log).
- **Reference ADR-0021** for mobile device attestation, which extends the identity model onto citizen devices.
- **SpiceDB stands** (earlier draft said Cedar; see `docs/spec-alignment/SA-005-authz-spicedb-vs-cedar.md`).
- **OpenBao stands** (earlier draft said Vault; see `docs/spec-alignment/SA-001-oss-hygiene.md`).

### ADR-0008 (Portal stack): amendments

- **Accessibility floor tightened to WCAG 2.2 AA** (was 2.1 AA per initial spec; MOBILE-INTEGRATION-NOTES mandates 2.2).
- **Reference ADR-0021** for mobile-specific security posture.
- **Path B stands**; Next.js excluded (see `docs/spec-alignment/SA-002-portal-stack-non-nextjs.md`).

### ADR-0012 (Data topology): amendments

- **Three-site T1 topology → two IEBC DR sites**, one T1 cluster set per site (voter / consensus / audit). Total 6 cluster instances instead of 9.
- **Ninety-day online log retention** aligned to RFP (tender.txt Sec V; PRE_BID D13); permanent archive via MinIO tiered storage.
- **Read replicas** at both DR sites plus at each of the five regional edge clusters (ADR-0019) for the read paths that edges serve.

### ADR-0015 (Cross-service consistency & event bus): amendments

- **Event-source topic hot retention aligned to 90 days**, tiered to MinIO thereafter (was "infinite hot + MinIO after 30d"). RFP-driven.
- **Karapace stands** (see `docs/spec-alignment/SA-001-oss-hygiene.md`).
- **JSON + zstd stands** (earlier draft said Avro). Trade-off acknowledged: Avro is tighter and registry-integrated; JSON is human-debuggable and one-codec. Karapace supports both, so if Datacraft has existing Avro artefacts, we can adopt with lower cost than a general Avro migration; otherwise JSON stands. See `docs/spec-alignment/SA-007-event-bus-json-vs-avro.md`.
- **Per-aggregate topic granularity stands** (earlier draft said ~10 domain topics). Consumer isolation and blast-radius arguments. See SA-007.

### ADR-0016 (PII & retention): amendments

- **Add per-election-cycle KEK hierarchy** (see ADR-0005 amendment). KEK destruction at end of statutory retention (15y after election cycle) triggers cryptographic erasure of the entire cycle's Tier A data as a single action. **This is the only cryptographic-erasure primitive we adopt**; per-record erasure requests continue to use soft-delete + scheduled purge + rotating backup keys as decided.
- **Voter data ownership statement**: no third-party use permitted (RFP Sec V); enforced by data-egress policy + audit alerting on any Tier A read whose destination is outside IEBC-controlled infrastructure.

### ADR-0017 (Transparency log): amendments

- **Five-channel publication for STHs and Merkle roots**, per `sw/ARCHITECTURE.md` §17.2:
  1. IEBC public website (signed JSON endpoint).
  2. Public GitHub mirror repository (root hashes committed).
  3. **IPFS pinned in Kenya** (Kenyan-hosted IPFS nodes; sovereignty preserved).
  4. Daily **Kenya Gazette** print publication of daily anchor root.
  5. Daily **KBC Radio** hash bulletin (audible cross-check).
- **STH cadence unchanged**: 1s hot / 60s default / 5min batch. The 10-minute "public Merkle root" cadence in spec ARCHITECTURE §1.5 maps to a subset of the 5-minute batch STHs, cross-published to the five channels above.

### ADR-0018 (Write-path performance): amendments

- **Add explicit SLA: 60,000 concurrent RTS sessions** per RFP (tender.txt Sec V). Load-test target adjusts to sustain 60k concurrent sessions with per-session write rate at realistic bimodal peak (~1 write per session per second at peak).
- **Add rts-svc RTO ≤ 30 min / RPO ≤ 5 min** per `sw/ARCHITECTURE.md` §14.3.
- **Reference ADR-0022** for the full release + DR posture that operationalises these SLAs.

## What is NOT amended

- **ADR-0002 (Languages)**: Rust default holds. Python fallback for ML/adapters/glue. **Flutter for kits and mobile (unchanged)**, RFP-binding ~14,100 legacy Android device reuse argues for Flutter's better legacy support over Kotlin+Compose. The earlier draft's Kotlin note is not RFP-derived and is withdrawn. Additions (ONNX Runtime, Celery, BagIt as pinned tools) folded in without changing decision.
- **ADR-0003 (Data infra)**: Karapace + Valkey + OpenSearch (scoped) + Kafka + MinIO + Postgres + ClickHouse-deferred. All defended.
- **ADR-0009, 0010, 0011, 0013, 0014**: unchanged in substance.
- **ADR-0001**: unchanged.

## New ADRs added by this pass

- **ADR-0019**: Regional edge topology (5 clusters).
- **ADR-0020**: Service topology (18 K8s services grouping 69 modules).
- **ADR-0021**: Mobile security posture (Play Integrity + App Attest + RASP).
- **ADR-0022**: Release process + DR (blue-green + 15-min rollback, RTO/RPO SLAs).

## Spec-alignment notes (edits to apply to sw/)

Six notes recording where the earlier `sw/` draft differs from the ADRs and which `sw/` sections to edit, tracked in `docs/spec-alignment/`:

- **SA-001, OSS hygiene** (OpenBao / Valkey / Karapace over Vault / Redis / Confluent SR).
- **SA-002, Portal stack non-Nextjs** (Path B defence citing CVE record).
- **SA-003, Kenya-sovereign edge** (surface spec's own §17.2 self-consistency argument).
- **SA-005, Authz engine SpiceDB vs Cedar** (relational-permissions fit).
- **SA-006, Mesh Cilium vs Linkerd** (operational consolidation).
- **SA-007, Event bus JSON + per-aggregate topics vs Avro + domain topics** (consumer-isolation argument).

(No note for language tiering: Rust default per ADR-0002 stands; the `sw/` Python-3.13-default text is simply corrected.)

## Outstanding items

IEBC-side questions in `docs/eel-clarifications-outstanding.md`; Datacraft bid-document backlog in `docs/datacraft-internal-backlog.md`. Highlights:

- DATACRAFT_SOFTWARE_SCOPE.md v2 covers only M01-M28 while module set is M01-M69.
- ACCEPTANCE-CRITERIA.md has no OAT/FAT criteria for M35-M69.
- API-CATALOG.md covers only M01-M28.
- Voter register retention contradiction (5y in DATA-MODEL vs 15y statutory).
- Kit language stale text (Flutter mentions after "Kotlin resolved" stitch).
- HSM level stale text in expect.md.
- CI in air-gapped path, ARCHITECTURE §14.1 names GitHub Actions / GitLab CI; contradicts air-gap mandate.
- Escrow line contradiction, tender explicitly says NOT required (L9556, L11377), yet Datacraft's plan referenced escrow.

## Consequences

Positive: single audit-trail document; original ADRs remain immutable-after-acceptance; every amendment is traceable to a specific spec section or RFP line; spec-alignment notes list the exact `sw/` edits; IEBC questions are separated from our own backlog.

Negative: readers must consult both the original ADR and this reconciliation to see the current state; mitigated by cross-references in the ADR index (`docs/adr/README.md`) marking amended ADRs.

## Related

All of ADRs 0002-0022. Spec-alignment notes under `docs/spec-alignment/`. `docs/eel-clarifications-outstanding.md`, `docs/datacraft-internal-backlog.md`. Decision log `docs/architecture/decision-log.md` Phase 14 (RFP-alignment reconciliation).
