# System Architecture — IEBC IEMS
## Full 69-Module Service Topology

**Companion to:** `PRD.md`, `DATACRAFT_SOFTWARE_SCOPE.md`
**Version:** 3.0
**Status:** Bid-stage design; final architecture converges at Preliminary Project Plan approval.
**T-0 (election day):** 2027-08-10

---

## 1. Architectural Principles

1. **Sovereign by design.** All production data resides in Kenya. Offshore access only via IEBC-controlled bastion with session recording and transparency-log entry.
2. **Zero-trust internal.** No implicit trust between services. Every service-to-service call uses mTLS (SVID from SPIFFE/SPIRE) + short-lived OPA-validated tokens. No shared credentials.
3. **Offline-first devices.** BVR, BVI, RTS, and all six field mobile apps operate fully without network connectivity, with local encrypted queue and signed submission on reconnect.
4. **Immutable infrastructure.** Every service runs from a signed OCI image. Deployments replace, never modify. GitOps via ArgoCD; all changes signed and logged.
5. **Cryptographic transparency.** Every state-change event is signed, countersigned, timestamped, hash-chained, and Merkle-logged with public roots every 10 minutes.
6. **Static-first public read.** Public Results Portal (M11), IEBC Verify App (M29), Referendum Explainer (M68), and Public Verifier (M13) serve signed static snapshots from CDN. Origin DDoS cannot stop citizen verification.
7. **Fail visibly.** Never silent failure. Every degradation surfaces to M16 ECC and SLO dashboards. Services expose structured health probes.
8. **Two-person integrity.** No single account can move critical state. Register edits, CA operations, deployments, DR failover, and result declarations require quorum approval enforced by OPA policy engine.
9. **Portability.** Standards-compliant biometric formats (WSQ, JPEG-2000, ISO 19794, ICAO 9303, ANSI/NIST-ITL). No proprietary lock-in on any data store or runtime.
10. **Single source of truth per domain.** Each data domain has exactly one authoritative service. All other services are consumers. Cross-domain writes go through the owning service's API — never direct database access.

---

## 2. Module-to-Service Mapping

The 69 modules compose into 18 deployable Kubernetes services. Grouping principle: shared data store + always co-deployed + synchronous happy-path communication = same service. Different SLOs, scaling profiles, or security boundaries = separate service even within a domain.

| Service | Modules | SLO (availability) | Scaling profile |
|---|---|---|---|
| **biometric-edge** | M01, M05, M08, M27, M31 | Not K8s — kit APK | Kit-side; deployed via MDM |
| **bvr-svc** | M02, M35, M36, M53, M64 | 99.99% (registration window) | 3–20 replicas, HPA |
| **abis-svc** | M03 | 99.9% | GPU node pool; confidential-compute enclave |
| **vprs-svc** | M04, M07 | 99.9% | CDN + 2–10 replicas |
| **bvi-svc** | M06 | 99.99% (polling window) | 5–40 replicas, HPA |
| **rts-svc** | M09, M10, M51, M62 | 99.99% (polling window) | 20–60 replicas, HPA; sharded by constituency |
| **portal-svc** | M11, M13, M29, M68 | 99.99% read (CDN-backed) | CDN-first; 2–8 replicas for API |
| **crms-svc** | M12, M22, M45, M46, M47, M67 | 99.5% (episodic) | 1–10 replicas, HPA |
| **transparency-svc** | M14, M15, M21, M57, M58, M59 | 99.9% | 2–8 replicas; read-only |
| **ecc-svc** | M16, M17, M44, M60 | 99.99% (election period) | 3–15 replicas; WebSocket fan-out |
| **boundary-svc** | M18, M52 | 99.5% | 2–4 replicas; high read-cache |
| **queue-svc** | M19 | 99.9% (election day) | 3–20 replicas; 100K updates/min peak |
| **inventory-svc** | M20, M39, M63, M65 | 99.99% (election window) | 3–15 replicas |
| **governance-svc** | M23, M24, M25, M40, M48, M49, M50 | 99.5% | 2–10 replicas; episodic load |
| **ai-platform-svc** | M26, M28, M66 | 99.5% | GPU node pool; batch + streaming |
| **field-mobile-svc** | M30, M32, M33, M34 | 99.9% (election period) | 5–80 replicas; 250K concurrent (M33) |
| **training-svc** | M37, M38, M41, M42, M43 | 99% | 1–6 replicas; episodic |
| **support-admin-svc** | M54, M55, M56, M61, M69 | 99% | 1–5 replicas; low throughput |

---

## 3. Sites and Deployment Topology

### 3.1 Primary Data Centre

- Hosted on IEBC-provided converged infrastructure (Anniversary Towers area, Nairobi).
- Active for all read and write traffic.
- Physical security: biometric entry, mantraps, 24x7 guard, environmental sensors, 72-hour generator + UPS, Novec 1230 fire suppression.
- Network: dual-carrier internet (Safaricom + Liquid Telecom) with BGP diversity, upstream DDoS scrubbing (Cloudflare Magic Transit or Akamai Prolexic), WAF at edge.

### 3.2 Mirror Data Centre (DR Site)

- Hosted on IEBC-provided infrastructure at a geographically separated site.
- Synchronous replication for write-critical paths: voter register, results, HSM signing operations, kit certificate store.
- Asynchronous replication for BI marts, audit warehouse, object store.
- Active-active for read traffic; active-passive for writes. Failover on primary loss.
- RTO: ≤ 30 min for rts-svc write path; ≤ 4 h for bvr-svc bulk enrolment. RPO: ≤ 5 min for rts-svc; ≤ 15 min for bvr-svc.

### 3.3 Five Regional Edge Nodes

- Kubernetes edge clusters per region (Nairobi, Mombasa, Kisumu, Eldoret, Garissa).
- Kit gateway, register bundle cache, BVI session cache, M33 dispatch pre-cache.
- Reduces kit-to-back-end round-trip from 200ms (national DC) to 30-60ms (regional edge).
- Fail-forward: edge cluster failure routes kit traffic to primary DC with graceful degradation.

### 3.4 Escrow Mirror

- Read-only at an independent custodian (proposed: Office of the Auditor-General or designated academic institution).
- Receives signed snapshots + Merkle transparency-log roots via async replication.
- Never receives write access. Court-order response can be served from escrow without granting operational-system access.

### 3.5 Air-Gapped Environments

- **PKI ceremony room:** Kit prep, root CA operations, key ceremonies. No internet. One-way data diode for signed image exports.
- **M39 printer environment:** Ballot printing chain receives RFID manifests via one-way data diode. No network path from printer floor to IEMS.

### 3.6 Deployment Zones per Service

| Zone | Services |
|---|---|
| Kit-side (biometric-edge) | M01, M05, M08, M27, M31 — Android APK, not K8s |
| Edge cluster (5 regional) | Kit gateway, bvr-svc read cache, bvi-svc session cache, rts-svc ingress buffer |
| Primary + Mirror DC (Kubernetes) | All 17 K8s services |
| CDN edge | portal-svc static snapshots (M11, M13, M29, M68) |
| Air-gapped provisioning | PKI ops, kit cert issuance |

---

## 4. Service Dependency Graph

Arrows denote runtime dependencies (A → B means A calls B synchronously in the happy path). Kafka-mediated async relationships shown separately in §9.

```
biometric-edge → bvr-svc (M01 enrolment submission)
biometric-edge → bvi-svc (M05 poll-book events)
biometric-edge → rts-svc (M08 form submission)
biometric-edge → boundary-svc (M05 geofence query)

bvr-svc  → abis-svc (ABIS 1:N trigger)
bvr-svc  → boundary-svc (electoral-area validation)
bvr-svc  → vprs-svc (pre-registration merge)

abis-svc → bvr-svc (match result + dedup flag)

vprs-svc → bvr-svc (pre-reg handoff)

bvi-svc  → boundary-svc (geofence polygon)
bvi-svc  → ecc-svc (ops dashboard feed)

rts-svc  → portal-svc (results publication trigger)
rts-svc  → transparency-svc (signed form events)
rts-svc  → ecc-svc (transmission status feed)
rts-svc  → governance-svc (M40 evidence package trigger)

portal-svc → transparency-svc (Merkle root fetch)

crms-svc → bvr-svc (candidate voter-register cross-check)
crms-svc → boundary-svc (electoral-area list)
crms-svc → rts-svc (ballot-form list for ICR model)

transparency-svc → (read-only Kafka consumer; no upstream K8s calls)

ecc-svc  → all services (health-probe fan-out; read-only)
ecc-svc  → field-mobile-svc (M33 dispatch command)

boundary-svc → (read-only; consumed by all; no upstream calls)
queue-svc → ecc-svc (s.14A alert trigger)

inventory-svc → bvr-svc (kit-cert binding)
inventory-svc → crms-svc (ballot UID handoff)
inventory-svc → rts-svc (ballot-serial reconciliation)

governance-svc → bvr-svc (voter register query for M48 recall)
governance-svc → crms-svc (nomination query for M40)
governance-svc → rts-svc (referendum result query for M49)
governance-svc → inventory-svc (diaspora container query for M65)

ai-platform-svc → rts-svc (anomaly flags)
ai-platform-svc → ecc-svc (anomaly alerts)
ai-platform-svc → transparency-svc (M66 forensics read)

field-mobile-svc → bvr-svc (M31 voter-lookup backup)
field-mobile-svc → inventory-svc (M32 spare-parts lookup)
field-mobile-svc → ecc-svc (M33 attendance feed)
field-mobile-svc → rts-svc (M30 Form 34A verification)

training-svc → boundary-svc (electoral-area reference)
training-svc → field-mobile-svc (M38 certification → M33 dispatch eligibility)

support-admin-svc → bvr-svc (M64 voter-list redacted view)
support-admin-svc → inventory-svc (M69 archival trigger)
support-admin-svc → rts-svc (M62 transmission audit log)
support-admin-svc → field-mobile-svc (M55 payroll trigger from M33)
```

**Transitive dependencies:** abis-svc has no outbound K8s calls other than bvr-svc. This isolation is intentional: the confidential-compute enclave containing unencrypted biometric templates must have the smallest possible blast radius.

---

## 5. Multi-Tenancy Model

The IEMS is a single-tenant system (one IEBC) but has internal multi-tenancy axes:

| Axis | Model | Implementation |
|---|---|---|
| Election cycle | Each general election, by-election, and referendum is a separate `election_cycle_id` partitioned in PostgreSQL; prior-cycle data is read-only | Row-level security in PostgreSQL; cycle ID injected into OPA context on every request |
| County | Each county's data is logically isolated within shared services; county ICT officers see only their county | OPA policy binds county ICT role to county_id claim in SVID; enforced at service layer |
| Party (CRMS) | Each party's nomination data is isolated; party portal users see only their party | Keycloak party_id claim; OPA enforced at crms-svc |
| Observation mission | Each observer mission (EU EOM, Carter Center, AU) sees only mission-scoped data | API key scope in M57; Rego policy per mission organisation type |
| IAB vs. ECC | IAB reads from a separate Kafka topic with immutable, verified events; ECC reads from the same topic but also has write paths | Separate K8s namespaces; NetworkPolicy and AuthorizationPolicy block IAB namespace from write endpoints |

---

## 6. Service Mesh Topology

**Service mesh:** Linkerd 2 (stable channel). All pod-to-pod traffic inside the primary and mirror clusters uses mTLS via SPIFFE SVIDs automatically managed by SPIRE.

```
SPIRE Server (primary + mirror, HA)
  |-- SPIRE Agent (DaemonSet on every node)
  |     |-- Issues SVID (X.509 cert, 1h TTL) to every pod
  |
Linkerd control plane
  |-- Automatic mTLS on every meshed deployment
  |-- Per-namespace AuthorizationPolicy:
  |     - bvi-svc can only receive from biometric-edge and boundary-svc
  |     - abis-svc can only receive from bvr-svc
  |     - transparency-svc has no inbound from production services (Kafka-only)
  |-- Linkerd Viz for observability (exported to Prometheus)
```

**Key mesh policies:**

- `abis-svc` is in a dedicated namespace with a policy that blocks all inbound except from `bvr-svc`. This prevents any service from querying biometric templates directly.
- `transparency-svc` is in a read-only namespace; its policy blocks all write-path endpoints from within the mesh.
- `ecc-svc` health probes reach all services via a dedicated ServiceAccount with GET-only RBAC; it never writes to any service.
- All cross-service calls carry a `X-Correlation-ID` header propagated from the originating request; Linkerd injects span context via the B3 multi-trace format.

---

## 7. Failure Isolation Boundaries

| Boundary | What fails | What survives | Recovery |
|---|---|---|---|
| abis-svc pod failure | New dedup 1:N queries | All BVR kit operations (queue and submit without dedup result); BVI polling-day identification (local match on kit) | Kubernetes pod restart; ≤60s recovery |
| bvi-svc unavailable | Polling-day back-end event logging | Kit-side auth continues offline; poll-book events queued locally | Async replay on recovery |
| rts-svc primary pods down | New result submissions temporarily queued at edge | All previously submitted results intact in PostgreSQL; portal serving cached snapshot | HPA + pod restart; mirror DC active for reads |
| portal-svc origin DDoS | Origin API calls | CDN-cached signed snapshots; citizen verification continues | CDN serves stale-but-valid snapshot; origin auto-scales behind CDN |
| ecc-svc down | War-room dashboard | All operational modules (bvr-svc, bvi-svc, rts-svc) continue independently | ECC restart; state reconstructed from Kafka replay |
| boundary-svc down | Geofence queries degrade gracefully | Kit uses cached polygon (last-known); human supervisor override with audit flag | Redis cache miss falls through to read replica |
| Kafka broker failure (1 of 3) | Throughput degrades | Event bus continues with remaining brokers; no event loss (replication factor 3) | Kafka auto-leader-election; ≤30s |
| Primary DC power failure | Write traffic | Mirror DC activates (active-active for reads already); RTO ≤30 min for write path | Patroni failover for PostgreSQL; ArgoCD reconciles on mirror cluster |
| Single regional edge node failure | Local kit round-trip latency | Kits fail-forward to primary DC; ~150ms latency increase | Edge cluster pod restart; ≤5 min |
| HSM appliance failure | Signing operations | HSM cluster (3× redundancy) continues; queued signing completes when failed unit replaced | Cluster serves 3× peak capacity |

---

## 8. Data Flows

### 8.1 Voter Registration Flow (bvr-svc domain)

```
[Citizen] → [Enrolment Clerk] → [KIEMS BVR device (M01)]
  |  local dedup, biometric capture, on-device signing
  |  offline queue if no network (SQLCipher Room DB)
  |
  [Telco VPN → mTLS → Regional Edge Kit Gateway]
  |
  [bvr-svc M02 (ingest → staging)]
  |  digital certificate check; event to Kafka: bvr.enrollments
  |
  [abis-svc M03 (1:N dedup against 30M records, confidential enclave)]
  |
  [No match: enrol]  [Match: adjudication queue]  [Quality fail: reject + retry]
       |
  [Voter Register (PostgreSQL 17 primary)]
       |
  [Merkle builder → Transparency log → Escrow mirror]
       |
  [Kafka outbox → BI mart (async)] → [IEBC Power BI]
       |
  [M35 CVR: life-event continuous maintenance]
  [M36: deregistration appeal if voter removed]
  [M53: voter ID card production trigger]
  [M64: voter-list public inspection (T-90 to T-60)]
```

### 8.2 Polling Day — BVI Flow (bvi-svc domain)

```
[Voter] → [Presiding Officer] → [KIEMS BVI device (M05)]
  |  fingerprint/iris match against local register bundle
  |  geofence check via boundary-svc (cached polygon)
  |  poll-book event signed by kit private key
  |  offline capable: events queued in SQLCipher
  |
  [bvi-svc M06 — poll-book event ingestion]
  |  Kafka: bvi.events → M44 turnout analytics → M19 queue transparency
  |  Kafka: bvi.events → M16 ECC dashboard
  |  No biometric data transmitted post-identification
  |
  [M31 Mobile Poll Clerk: companion reference + incident logging (same device)]
```

### 8.3 Results Transmission Flow (rts-svc domain)

```
[KIEMS RTS device (M08)]
  |  Form 34A scanned (camera + perspective correction + M27 quality gate)
  |  PDF/A generated with per-form digital signature (kit private key)
  |  QR embed: SHA-256 hash of PDF
  |  randomised jitter (0-10 min) before transmission
  |
  [Telco VPN → Regional Edge → rts-svc M10]
  |  parallel streams:
  |    ICR/ONNX extraction → tally proposal (advisory)
  |    Form 34A image → MinIO (immutable)
  |    Kit signature verification
  |    Kafka: rts.forms → M26 anomaly detection, M14 Merkle builder, M15 IAB
  |
  [M09 Returning Officer tally module]
  |  human review of ICR extraction; agent + auditor counter-sign
  |  Form 34B compiled; HSM-signed by RO
  |
  [M51 Result Declaration + Certification]
  |  Form 34C/D Commissioner digital signature (HSM, dual-Commissioner for presidential)
  |  gazette-ready PDF/A generated
  |
  [portal-svc M11]
  |  signed snapshot pushed to CDN every 5 minutes
  |
  [Transparency log + Escrow mirror]
  |
  [portal-svc M13 / M29: any citizen verifies Merkle inclusion proof]
  |
  [M62 transmission legal-basis audit: every submission logged with DPA purpose code]
```

### 8.4 Ballot Physical Chain Flow (inventory-svc domain)

```
[Al Ghurair printer MES] → [M39 Ballot Printing Chain]
  |  RFID chip manifest ingested via one-way data diode
  |  QC scan at printer: defect rate, roll-level tracking
  |
  [M20 Inventory Management: warehouse receiving]
  |  ballot bundle UID assigned; Merkle custody node created
  |
  [M63 Ballot Serial Chain: warehouse → polling station]
  |  serial-range assignment per station at kit-prep time (T-30)
  |  station assignment signed by IEBC Ops officer
  |  spoilt/replaced/reconciled dispositions tracked per paper
  |
  [M08 RTS device: reconciliation report at station close]
  |  M63 validates against assigned serial ranges
  |
  [M40 Election Court IT Interface: petition evidence]
  |  serial-range commitments published in transparency log
  |  range-membership proof for any individual ballot
  |
  [M65 Diaspora Ballot Physical Logistics: return leg]
  |  tamper-evident container → customs → Nairobi tally centre
  |  chain-of-custody handoff signed at each hub
```

### 8.5 Field Operations Flow (field-mobile-svc + support-admin-svc)

```
[M55 HR/Payroll: worker engagement + certification gate (M38)]
  |
  [M33 Personnel Dispatch: push assignment, GPS check-in]
  |  M16 ECC sees attendance feed in real time
  |
  [Election Day]
  |  M30 Party Agent: Form 34A counter-sign
  |  M31 Poll Clerk: companion reference (same kit)
  |  M32 Field Engineer: remote diagnostics, M20 spare-parts
  |  M34 Observer: incident report + Merkle verification
  |
  [Station Close]
  |  M33 GPS check-out → M55 payroll trigger (M-Pesa bulk)
  |  M60 Medical: incident triage, ambulance dispatch
  |
  [M41 Post-Election Review: findings registry → M69 archive]
```

---

## 9. Event Bus and Streaming Topology

**Event bus:** Apache Kafka 3.x + Confluent Schema Registry (Avro schemas). All events use CloudEvents 1.0 envelope (JSON). Kafka retention: 7 days hot; permanent to MinIO (compacted topic export).

**Kafka topic taxonomy:**

| Topic | Producers | Consumers | Partitioning |
|---|---|---|---|
| `iebc.bvr.enrollments` | M01 (via bvr-svc ingest) | abis-svc, M35 CVR, P05 audit | By constituency_id (290 partitions) |
| `iebc.bvi.events` | M05 (via bvi-svc) | M44 turnout, M19 queue, M16 ECC, M26 anomaly | By station_id (55,393 partitions, Kafka broker-managed segment) |
| `iebc.rts.forms` | M08 (via rts-svc) | M10 national agg, M14 Merkle, M15 IAB, M21 RLA, M26 anomaly, M62 audit | By constituency_id |
| `iebc.rts.tallies` | rts-svc M10 | portal-svc M11/M13/M29, M15 IAB, M44, M40 evidence | By election_cycle + position_id |
| `iebc.inventory.custody` | inventory-svc M20/M39/M63/M65 | M16 ECC, M40 evidence, P05 audit | By kit_uid_prefix (hash) |
| `iebc.calendar.events` | boundary-svc M52 | All 17 K8s services (cache invalidation) | Single partition (low volume, strict ordering) |
| `iebc.governance.incidents` | governance-svc M23/M48-50, field-mobile-svc M30/M34 | M16 ECC, M15 IAB, M40 evidence | By incident_severity_bucket |
| `iebc.ai.anomalies` | ai-platform-svc M26 | M16 ECC, M15 IAB, M40 evidence | By anomaly_type |
| `iebc.audit.transmissions` | All services (via M62 SDK) | support-admin-svc M62 ClickHouse sink | By service_name |
| `iebc.field.attendance` | field-mobile-svc M33 | M16 ECC, support-admin-svc M55 | By county_id |

**Exactly-once semantics:** rts.forms and inventory.custody topics use Kafka transactions (idempotent producer + transactional consumer) to prevent duplicate tally or duplicate custody records. Other topics use at-least-once with idempotency keys in the event payload.

**Kafka cluster sizing:** 15 broker nodes (3 per region × 5 regions for edge + 6 in primary DC + 3 in mirror DC). Replication factor 3 for all critical topics. Peak throughput: 100K events/second (bvi.events during morning station-open rush, all 46K stations active simultaneously).

---

## 10. API Gateway Design

**API gateway:** Envoy Proxy (data plane) managed by Contour ingress controller, with Kong Gateway as the control plane for rate limiting, key management, and plugin extensibility.

**Two gateway tiers:**

**Tier 1 — Public edge gateway** (faces internet):
- Routes to: portal-svc (M11, M13, M29, M68), vprs-svc (M04, M07), governance-svc (M23 grievance intake)
- Controls: rate limiting per IP (10 req/s citizen lookups, 100 req/min grievance intake), CORS, WAF (OWASP Core Rule Set v4), DDoS detection, bot mitigation (Turnstile/hCaptcha on write endpoints)
- Authentication: anonymous (public portal), OAuth2 PKCE (citizen self-service)
- TLS termination: TLS 1.3; HSTS; certificate pinning for mobile apps

**Tier 2 — Authenticated internal gateway** (faces IEBC operators, party agents, field officers):
- Routes to: crms-svc, bvr-svc admin, transparency-svc, ecc-svc, training-svc, support-admin-svc
- Controls: FIDO2/OTP MFA enforced at Keycloak; short-lived JWT (15-min expiry); per-role rate limits; per-tenant namespace routing
- Authentication: Keycloak OIDC; SPIFFE SVID for service-to-service
- Audit: every authenticated request logged with 5-tuple (who, what, which entity, from where, when) via M62 SDK

**Kit gateway** (separate from both tiers):
- Dedicated mTLS endpoint for KIEMS kits; kit identity from per-kit X.509 certificate
- No Keycloak involvement; certificate-based auth only
- Rate limiting per kit (100 req/min normal; 1,000 req/min burst for thundering-herd at station close)
- Separate Kubernetes Deployment in each regional edge cluster

---

## 11. Read/Write Separation

| Service | Write path | Read path |
|---|---|---|
| bvr-svc | PostgreSQL primary (primary DC); synchronously replicated to mirror | PostgreSQL read replicas (2 per DC); Redis (session cache) |
| bvi-svc | PostgreSQL primary (poll-book events) | Redis (station register bundle; pre-loaded) |
| rts-svc | PostgreSQL primary (sharded by constituency_id); ICR worker pool | PostgreSQL read replicas; Redis (session auth cache); CDN (portal snapshots) |
| portal-svc | No direct writes; consumes Kafka and pushes to CDN | CDN edge (static snapshots); Redis (result cache TTL 60s) |
| inventory-svc | PostgreSQL primary (kit custody events, Merkle chain) | PostgreSQL read replica; Redis |
| transparency-svc | Kafka consumer only (append-only event log); ClickHouse (audit warehouse) | ClickHouse (audit queries); OpenSearch (evidence search) |
| ecc-svc | PostgreSQL primary (incident records, dispatch commands) | Redis Pub/Sub (WebSocket fan-out to operator dashboards) |
| ai-platform-svc | ClickHouse (ML telemetry, anomaly log) | ClickHouse (analytics queries) |

**Write amplification:** Every write to the PostgreSQL primary also: (1) publishes a Kafka event via Debezium CDC outbox; (2) updates a Redis cache TTL (where applicable); (3) eventually creates a Merkle leaf in the transparency log (batched every 10 minutes). This triple-write pattern is explicitly chosen for auditability over write efficiency.

---

## 12. Caching Topology

Five caching layers:

| Layer | Technology | What is cached | TTL |
|---|---|---|---|
| CDN edge (public read) | Cloudflare or Akamai | Signed portal snapshots (M11, M13, M29, M68); voter-education content (M37) | 5 min (election day); 1 hour (inter-election) |
| Service-level Redis (per service) | Redis 7 Cluster | Session tokens, rate-limit counters, result cache, station register bundles, geofence polygons | 15 min (session); 5 min (results); 24 h (register bundle) |
| Kit-side SQLite (offline) | SQLCipher-encrypted Room DB | Station register bundle, pending submission queue, reference materials | Refreshed at kit-prep; delta updates on connectivity |
| Read replica (PostgreSQL) | PostgreSQL 17 streaming replica | Register reads, results reads, inventory reads | Replica lag target ≤ 1 s |
| In-process LRU (Rust hot path) | Custom Rust LRU in HSM signing wrapper | HSM key references (not keys); avoiding PKCS#11 round-trip for every sign | 60 s (key reference TTL) |

**Cache invalidation:** All cache invalidations are event-driven via the `iebc.calendar.events` Kafka topic (for date-dependent data) or Debezium CDC events (for record-level invalidation). No TTL-only invalidation on critical paths; stale cache is always a degradation mode, never a correctness mode.

---

## 13. Sync and Async Communication Patterns

| Pattern | When used | Implementation |
|---|---|---|
| Synchronous REST (OpenAPI 3.1) | Read queries, admin actions, intra-service calls requiring immediate response | Envoy gateway; HTTP/2 + gRPC where high throughput needed |
| Synchronous gRPC | High-throughput kit ingestion (rts-svc form upload, bvi-svc poll-book events at 60K concurrent) | Protobuf schemas; bidirectional streaming for kit session |
| Async Kafka (CloudEvents) | All audit events, cross-service fan-out, long-running workflows | CloudEvents 1.0 envelope; Avro schema registry; dead-letter topic for retry |
| WebSocket (server push) | ECC real-time dashboard (M16), IAB live feed (M15), M44 turnout live chart | Redis Pub/Sub → WebSocket broadcast; connection multiplexed per operator session |
| Webhook (inbound) | M-Pesa payment confirmation (M47, M55), WhatsApp bot (M13), printer MES events (M39) | Signed request verification; idempotency key on every webhook; replay within 24 hours |
| Offline queue (kit-side) | All kit submissions: BVR enrolments, BVI poll-book events, RTS form submission, M33 attendance | SQLCipher queue; per-record UUID7 idempotency key; submit-and-forget with signed receipt |
| Batch export | M69 archive ingestion, M59 OAG financial extracts, M66 forensics data lake loads | Async job via Celery (Python); BagIt packaging for M69; CSV/JSON for M59/M66 |

---

## 14. Deployment Pipeline Architecture

### 14.1 CI/CD Overview

```
Developer pushes to feature branch
  |-- GitHub Actions / GitLab CI
  |    |-- Lint (ruff + mypy for Python; detekt for Kotlin; dart analyze for Flutter)
  |    |-- Unit tests (pytest -x; JUnit; dart test)
  |    |-- SAST (Semgrep, Bandit, SpotBugs)
  |    |-- Dependency audit (pip-audit, OWASP Dependency-Check, Snyk)
  |    |-- Secret scanning (truffleHog, GitGuardian)
  |    |-- Contract tests (Pact for each service pair)
  |    |-- M62 audit-SDK coverage check (blocks PR if <100%)
  |    |-- Container image build (multi-stage Dockerfile, distroless base)
  |    |-- Container image signed (cosign + Sigstore)
  |    |-- Push to IEBC internal Harbor registry
  |
On merge to main:
  |-- Integration tests (Playwright + k6 smoke test against kind cluster)
  |-- SBOM generation (CycloneDX, pushed to Harbor OCI artifact)
  |-- ArgoCD: automatic sync to Staging environment
  |
On tag (release/vX.Y.Z):
  |-- Canary rollout: 1% → 5% → 25% → 100% (automated regression gate)
  |-- ArgoCD: sync to Production on IEBC Commissioner approval (GitOps PR merge)
  |-- Promotion event written to P05 audit log + transparency log
```

### 14.2 Immutable Infrastructure

- Every service runs from a signed OCI image. Deployments use `Recreate` strategy for stateful services (bvr-svc, abis-svc) and `RollingUpdate` (maxSurge 1, maxUnavailable 0) for stateless services.
- No `kubectl exec` in production; all operational changes go through ArgoCD. Any manual `kubectl` command on production generates an alert via the audit webhook.
- Node images are built by Datacraft CI/CD, signed, and validated by cluster admission control (cosign policy) before any pod can schedule.
- Configuration is entirely in Git (Helm values + Kustomize overlays). No ad-hoc `kubectl apply`. Every config change has a PR + dual-review + signature.

### 14.3 Blue-Green for Zero-Downtime

For services with state-schema changes (bvr-svc, rts-svc after a migration):
1. Migration applied with Alembic `--sql` mode (generates SQL, reviewed in PR, applied manually by two DBA engineers).
2. Migration is always backward-compatible (new column added as nullable; old column deprecated, not dropped, for 2 release cycles).
3. Blue environment (current) continues serving while Green (new) starts.
4. Traffic shifted 0% → 5% → 25% → 100% to Green via Kong weighted upstream.
5. Blue kept hot for 15 minutes post-cutover; rollback in ≤ 90 seconds if regression detected.

### 14.4 Kit APK Deployment

Kit-side APKs (biometric-edge: M01, M05, M08, M27, M31) are deployed via MDM (M20 inventory + OTA pipeline):

1. APK signed by Datacraft code-signing certificate (Intermediate CA: Code-signing).
2. APK manifest (SHA-256 hash per APK per kit-SKU) committed to Git and pushed to ArgoCD.
3. MDM deployment job reads manifest; pushes APK delta to regional edge node.
4. Kit pulls delta on next heartbeat; verifies signature before installation.
5. Installation confirmed to M20 inventory; confirmation signed by kit TPM.
6. Rollback: previous APK version retained on kit; MDM can roll back in ≤ 5 minutes.

**Software freeze applies to APKs:** T-30 freeze means no new APK push except dual-approved critical security patches. Every post-freeze change requires Commissioner dual-approval + CCB ratification + transparency-log entry + new Kenya Gazette hash publication.

---

## 15. Security Architecture

Full detail in `SECURITY.md`. Key controls at the architectural level:

### 14.1 Identity

- **Human:** Keycloak 23+ OIDC/OAuth2. FIDO2 hardware key mandatory for all privileged accounts (Commissioner, ICT Admin, RO). SMS MFA prohibited on privileged accounts.
- **Service:** SPIFFE/SPIRE. Every pod has a SPIFFE ID and short-lived SVID (X.509, 1-hour TTL, auto-rotated by SPIRE Agent). Linkerd 2 validates SVID on every inbound connection.
- **Device:** Per-kit X.509 certificate issued at kit-prep time from the IEBC intermediate CA (device sub-CA). Certificate bound to kit TPM/secure element. Revocation in ≤ 60 s via OCSP must-staple + CRL push via MDM.

### 14.2 PKI Structure

```
Root CA (air-gapped, HSM-resident, M-of-N quorum, Thales Luna 7)
  |
  |- Intermediate CA: Services (issues SVIDs via SPIRE)
  |- Intermediate CA: Devices (issues per-kit certs at kit-prep time)
  |- Intermediate CA: TLS (issues server certs for all IEMS domains)
  |    |- *.iebc.or.ke, *.iems.ke (wildcard SAN)
  |- Intermediate CA: Code-signing (signs KIEMS APKs, container images, reproducibility bundle)
```

Root CA ceremony: M-of-N (5-of-9) offline ceremony; ceremony hardware stored at two geographically separate IEBC facilities.

### 14.3 Cryptographic Profile

| Use case | Algorithm | Rationale |
|---|---|---|
| Long-lived result signatures (Form 34A-D) | ML-DSA (CRYSTALS-Dilithium, NIST FIPS 204) + ECDSA P-384 hybrid | Post-quantum resistance for signatures that must verify in 15+ years (M69 archive) |
| Short-lived signatures (API tokens, kit events) | ECDSA P-384 | Fast; well-audited; sufficient for tokens with ≤15-min validity |
| Key exchange (new session establishment) | X25519 + ML-KEM-768 (CRYSTALS-Kyber, NIST FIPS 203) hybrid | Post-quantum KEM for new sessions on sensitive channels |
| Symmetric encryption at rest | AES-256-GCM with envelope encryption | HSM-wrapped DEKs; biometric fields use field-level encryption with separate keys |
| Hash function | SHA-256 (operational) + SHA-3-256 (archive commitments in M69) | SHA-3-256 adds domain separation for long-lived archive commitments |
| HSM | Thales Luna 7 (or Utimaco CryptoServer), FIPS 140-3 Level 3+ | Keys generated and stored in HSM; never exported; dual-control for admin ops |

### 14.4 Zero-Trust Internal Network

- All services behind Linkerd 2 mTLS; no plaintext service-to-service communication.
- OPA policy engine evaluates every incoming request: source SVID, destination endpoint, operation, caller identity claims.
- Cedar (Amazon) for fine-grained resource-level decisions (which RO can access which constituency; which party agent can counter-sign which Form 34A).
- Kubernetes NetworkPolicy micro-segments namespaces: abis-svc namespace accepts inbound only from bvr-svc namespace.
- Break-glass admin procedure: requires dual-Commissioner activation; logged in transparency log within 60 seconds of activation; full session recording.

### 14.5 Confidential Computing

- abis-svc matching workers run inside AMD SEV-SNP or Intel TDX confidential VMs. Unencrypted biometric templates exist only within enclave memory during the match window.
- Remote attestation verified by bvr-svc before dispatching templates. Attestation evidence stored in M15 IAB audit feed.

---

## 15. Observability Architecture

**Three pillars:** logs, metrics, traces. One additional pillar: **events** (the Kafka audit trail is also the primary event telemetry).

| Signal | Source | Collection | Storage | Retention |
|---|---|---|---|---|
| Structured logs (JSON) | All 18 K8s services | Promtail (DaemonSet) | Loki | 90 days hot; Kafka topic for permanent archive |
| Metrics | OpenTelemetry collector sidecar on every pod | Prometheus scrape | Prometheus (30-day hot); Thanos for long-term | 90 days |
| Distributed traces | OpenTelemetry SDK in every service | OpenTelemetry collector | Tempo | 30 days |
| Audit events (tamper-evident) | M62 SDK on every service | Kafka `iebc.audit.transmissions` | ClickHouse (append-only, hash-chained) | Permanent |
| Kit telemetry | KIEMS heartbeat events | Kit gateway → Kafka `iebc.inventory.custody` | inventory-svc + M16 ECC | 7 years |

**SLO dashboards:**

| Service | SLO | Error budget |
|---|---|---|
| bvi-svc (polling window) | 99.99% availability; p99 auth ≤100ms | 52 min/year |
| rts-svc (polling window) | 99.99% availability; p99 form ingest ≤2s | 52 min/year |
| portal-svc (CDN-backed read) | 99.99% read availability | CDN failure budget separate |
| bvr-svc (registration window) | 99.99% availability | 52 min/year |
| abis-svc | 99.9% availability; p99 1:N query ≤8s | 8.7 hours/year |
| ecc-svc (election period) | 99.99% | 52 min/year |

Alerts routed via multi-channel: email (off-hours), SMS (P1/P2), WhatsApp (P1), PagerDuty (P1 — pages on-call SRE). All alerts correlate to a Kafka audit event.

---

## 16. Resilience Patterns

- **Circuit breakers** (Envoy cluster circuit breaker + Resilience4j in Python services) on every external dependency: IPRS, KRA, telcos, HSM, M-Pesa, WhatsApp, printer MES, KNADS. Fail-open on read-only advisory lookups; fail-closed on integrity-critical calls.
- **Retry with exponential backoff and jitter** on transient failures; idempotency keys (UUID7) on every mutating request.
- **Bulkheads** between services: rts-svc ingress thread pool cannot exhaust abis-svc matching threads. Separate connection pools per service pair (PgBouncer per schema).
- **Back-pressure** everywhere: rts-svc refuses-fast (429) when Kafka producer queue is full; never blackholes.
- **Optimistic concurrency** on voter register writes (PostgreSQL version counter); reject-on-conflict to prevent register race conditions.
- **Warm capacity headroom:** 25% baseline; HPA auto-scales to 200% peak; capacity-tested in dress rehearsal at T-60.
- **Chaos engineering:** weekly chaos experiments via Chaos Mesh from T-300; kill random pods, simulate network partition between primary and mirror, kill HSM primary.
- **Graceful degradation ladder** per service:
  1. Full feature set
  2. Reduced feature set (e.g., portal serves cached snapshot only)
  3. Read-only
  4. Static signed snapshot from CDN
  5. Offline (kit continues without back-end; queues locally)

---

## 17. Data Sovereignty Architecture

### 17.1 Scope

All production voter-register records, biometric templates, results data, audit logs, kit telemetry, and candidate data physically reside in Kenya on IEBC-provided infrastructure. This is a legal obligation under the Kenya Data Protection Act 2019 and an IEBC policy requirement. No production data is processed outside Kenya. No foreign-state actor has access to production systems.

### 17.2 Sovereignty Controls

| Control | Implementation |
|---|---|
| Data residency | All K8s clusters, PostgreSQL primaries, Kafka brokers, MinIO buckets, Redis clusters, HSMs, and ClickHouse nodes physically located in Kenya (IEBC DC primary + IEBC DR site) |
| CDN data residency | CDN serves signed static snapshots only — no voter PII, no biometric data, no results before formal publication. CDN edge nodes outside Kenya serve only pre-published signed data that is already public |
| Escrow mirror | Located in Kenya at designated independent custodian; court-order response without granting access to operational systems |
| Remote support access | All Datacraft, Mechrevo, or vendor remote access is via an IEBC-controlled bastion: per-session Commissioner approval; full session recording with cryptographic timestamp; no bulk data access; no unattended access; session duration capped at 4 hours; extension requires fresh approval |
| Transparency log | Five publication channels: IEBC public website, GitHub (Datacraft account under IEBC supervision), IPFS pinned node in Kenya, daily Kenya Gazette print, KBC Radio daily hash bulletin. No channel requires trusting Datacraft |
| Biometric data | Never transmitted outside Kenya. Biometric templates never leave the confidential-compute enclave in abis-svc. Kit-side biometric data encrypted before any transmission; back-end decrypts only inside enclave |
| AI models (M17, M26, M28) | Models trained on Kenya-local data; stored in MinIO on IEBC infrastructure; ONNX format ensures no proprietary inference runtime lock-in; model export capability vests in IEBC at handover |

### 17.3 Foreign-Vendor Software Supply Chain

Third-party software components (ABIS engine SDK, HSM firmware, ONNX Runtime, Kafka, PostgreSQL) are supply-chain-controlled:

- All container images pulled from an IEBC-internal registry (Harbor, mirroring approved upstream images); no live pull from Docker Hub or vendor registries in production.
- SBOM (Software Bill of Materials) in CycloneDX format generated at every build; published at T-30 for public scrutiny.
- Dependency pinning: all Python packages locked with content hashes in `requirements.lock`; all Kotlin packages pinned in `gradle.lockfile`; all Dart packages in `pubspec.lock`.
- ABIS engine SDK sourced only under a signed contract specifying audit rights; source-code escrow for the SDK deposited with IEBC.

### 17.4 Post-Handover Sovereignty

At Operational Acceptance, IEBC receives:
- Root CA custody: air-gapped HSM transferred to IEBC physical control.
- All Vault seal keys rotated; new Thales Luna 7 HSM seals held by IEBC Commissioner only.
- All Keycloak admin credentials rotated; Datacraft admin accounts removed.
- All GitHub repository ownership transferred to IEBC GitHub organisation.
- All DNS records (iebc.or.ke subdomains) confirmed under IEBC DNS provider control.
- M69 archive key material transferred: long-term archive signing keys held by IEBC and KNADS jointly.

---

## 18. Environment Strategy

| Environment | Purpose | Data | Access | Notes |
|---|---|---|---|---|
| **Production** | Live IEMS | Real voter + results data | JIT admin, dual-control, full audit | Kenya sovereign perimeter only |
| **Mirror (DR)** | Standby | Synchronous replica | Same as production; active-passive for writes | Failover ≤ 30 min |
| **Escrow** | Independent custodian | Signed snapshots + Merkle roots | Read-only, designated custodian | Court-order ready |
| **Staging** | Pre-production validation | Sanitised subset (never real biometrics) | Datacraft SRE + QA; IEBC ICT on request | Schema-identical to production |
| **Sandbox** | Dress rehearsal + operator training | Fully synthetic (synthetic 30M voter register) | IEBC operators + Datacraft training | All 69 modules; scale-accurate |
| **Development** | Datacraft build | Fully synthetic | Datacraft developers | Docker Compose local; Kubernetes kind for integration |
| **CI** | Test execution | Fully synthetic | CI system only (GitHub Actions or GitLab CI) | Per-PR test suites |

Production data **never** leaves production. Sandbox uses synthetic data only, validated by M61 secrecy-assurance testing pipeline.

---

## 18. Technology Choices (rationale)

| Choice | Rationale |
|---|---|
| Kubernetes 1.31 + containerd | Portability, immutable infrastructure, IEBC-teachable, strong Kenya ecosystem |
| Linkerd 2 service mesh | Simpler than Istio; Rust data plane (low latency overhead); SPIFFE-native; CNCF graduated |
| SPIFFE/SPIRE | Standard for workload identity; Kubernetes-native; integrates with Vault and Linkerd |
| PostgreSQL 17 | Battle-tested at scale; TLA+-verified consistency; rich extension ecosystem; IEBC-operable post-handover |
| Kafka 3.x | Durable event backbone; supports Merkle-log signing chain; well-understood operational model |
| Redis 7 Cluster | Session cache; rate limiting; Pub/Sub for ECC WebSocket; no persistence risk (all Redis data is recoverable from PostgreSQL) |
| ClickHouse | Purpose-built for append-only audit warehouse; 1B-row queries in seconds; open source |
| MinIO | S3-compatible on-prem object store; sovereign; IEBC-operable; no vendor lock-in |
| OpenSearch | Voter register search; evidence search; open source (Elasticsearch fork); Kenya-deployable |
| Python 3.13 async | Datacraft house standard; mature async ecosystem; FastAPI + Starlette; IEBC ICT-readable for handover |
| Rust | Latency-critical paths: HSM signing wrapper, Merkle library (compiled to WASM for M13/M29), per-kit cert validator; memory safety without GC pauses |
| Flutter/Dart | Single codebase for 7 apps (M01/M05/M08 kit-side + M29/M30/M31/M32/M33/M34 field mobile); strong offline-first primitives; Kenya has active Flutter developer community |
| Next.js 14 | SSR + SSG for web portals; static export for CDN-backed public portals (M11, M13, M29, M68); server-side rendering for accessibility |
| OPA (Open Policy Agent) | Declarative policies as code; two-person-integrity rules expressible in Rego; auditable; Cedar for resource-level decisions |
| HashiCorp Vault | Short-lived secrets; dynamic credentials; ODPC-relevant audit; Thales Luna HSM seal |
| Envoy + Kong | Envoy as data-plane (proven at Lyft/Airbnb scale); Kong as control-plane for rate limiting and key management |
| ArgoCD | GitOps; all changes signed; auditability for CCB |
| ONNX Runtime | ICR inference in rts-svc; on-device inference in M28 AI monitoring; open standard; hardware-independent |
| BagIt (Library of Congress) | Standard archival packaging for M69; supported by KNADS and major institutional archives |
| ML-DSA / ML-KEM (NIST FIPS 203/204) | Post-quantum algorithms for long-lived signatures and new session key exchange; standardised August 2024 |

All choices are open-source-first where practical, with commercial support subscriptions (Confluent for Kafka, Kong Enterprise for gateway). All choices avoid vendor lock-in and are reproducible by IEBC without Datacraft.

---

## 19. Handover Architecture

At Operational Acceptance, IEBC receives:

- **Source code** for all 69 modules and 18 deployable services under GCC 15.4.
- **Build reproducibility:** signed manifest + build scripts that produce byte-identical OCI images from source. M14 `make reproduce` bundle included.
- **GitOps state:** ArgoCD Application manifests that reconstitute all 18 services from scratch on any Kubernetes 1.31+ cluster.
- **Documentation:** This file, PRD.md, SECURITY.md, DATA-MODEL.md, API-CATALOG.md, ACCEPTANCE-CRITERIA.md, INTEGRATION-AUDIT.md, all 69 module PRDs, all 18 service runbooks, ADRs.
- **Credentials handover:** all secrets rotated at handover; held by IEBC alone; Vault root token ceremony.
- **PKI transfer:** root CA custody transfers to IEBC in a formal M-of-N ceremony; all intermediate CA keys rotated.
- **Sandbox:** complete sandbox environment (all 69 modules, synthetic 30M voter register, all field mobile apps in sandbox MDM) handed to IEBC for ongoing operator training, by-election drills, and referendum rehearsals.
- **Knowledge transfer:** certified training for 23 HQ ICT + 47 County ICT + 200+ field engineers.
- **Escrow continuity:** independent custodian relationship transferable to IEBC governance.
- **M69 archive:** long-term archive ingestion pipeline operational; KNADS relationship established; format-migration pipeline documented and tested.

The architecture ensures IEBC can operate, extend, audit, and archive the system without any Datacraft or Mechrevo dependency after handover.

---

## 20. Network Architecture Detail

### 20.1 Kit-to-DC Network Path

Each KIEMS kit connects via:

1. **Primary:** LTE/4G SIM (Safaricom or Airtel); routed through IEBC-negotiated APN to carrier VPN endpoint in IEBC DC.
2. **Secondary:** eSIM carrier rotation (carrier determined by signal quality; M-of-N consensus between three carriers before rotation decision).
3. **Fallback 1:** 3G / EDGE / HSPA on same SIM; reduced throughput but Form 34A image transmission still completes within 10 minutes.
4. **Fallback 2:** SMS channel for Form 34A tally numbers only (image stored locally; transmitted when connectivity restores). SMS payload signed and encrypted; back-end stores as provisional pending image delivery.
5. **Fallback 3:** LoRa mesh (where deployed in remote constituencies); kit relays via LoRa to a mesh gateway that has LTE; range ~5 km; throughput 250 bps (tally numbers only).
6. **Fallback 4:** Physical courier of kit to constituency office; image uploaded from constituency LAN.

All six tiers are pre-configured at kit-prep time. The kit selects the best available tier automatically. Tier selection is logged in M20 kit telemetry for post-election analysis.

### 20.2 IEBC DC Network Segmentation

```
INTERNET EDGE
  Cloudflare Magic Transit (DDoS scrubbing, anycast)
  IEBC BGP AS (dual upstream: Safaricom + Liquid Telecom)
    |
PERIMETER ZONE
  Edge load balancers (HA pair, ECMP)
  WAF (OWASP CRS v4, custom IEBC rules)
  Kong API Gateway (public tier)
    |
DMZ
  Kit gateway (carrier VPN termination; mTLS from kit certs)
  Portal CDN origin (Next.js SSR for portal-svc)
    |
INTERNAL ZONE (Kubernetes Cluster Network)
  Linkerd service mesh (mTLS between all pods)
  Namespace-level NetworkPolicy:
    - namespace/bvr    → ingress: kit-gw, vprs, abis, boundary
    - namespace/abis   → ingress: bvr only
    - namespace/rts    → ingress: kit-gw, crms, governance
    - namespace/portal → ingress: rts (read), transparency (read)
    - namespace/ecc    → ingress: all services (read-only health probes)
    - namespace/iam    → ingress: all services (OIDC token validation)
    |
DATA ZONE
  PostgreSQL 17 HA cluster (Patroni + HAProxy)
  Kafka cluster (15 brokers, rack-aware)
  Redis Cluster (6 nodes, 3 shards)
  MinIO distributed (8 nodes, erasure-coded)
  ClickHouse cluster (6 nodes, ReplicatedMergeTree)
  OpenSearch cluster (6 nodes)
    |
HSM ZONE (physically separate rack, locked)
  Thales Luna 7 cluster (3 nodes, A/A/A)
  PKCS#11 network HSM proxy (encrypted channel from data zone)
  Air-gapped management console (no network; USB-only for admin ops)
```

### 20.3 Carrier VPN Design

- Three independent VPN tunnels per kit: Safaricom, Airtel, Telkom Kenya.
- Kit uses the tunnel for the carrier whose SIM is active.
- VPN protocol: IPsec IKEv2 with ECDSA P-384 authentication (kit certificate as IKE identity).
- VPN concentrators at IEBC DC are hardware appliances (not software VPN); IEBC-owned; Datacraft integrates.
- BGP failover between carriers at IEBC DC in ≤ 30 seconds on carrier link failure.

---

## 21. C4 Model References

C4 model diagrams (context, container, component, code level) are maintained as interactive HTML in `../diagrams/architectural/`. Summary:

| C4 Level | File | Description |
|---|---|---|
| L1 — System Context | `a1-system-context.svg` | All external actors (voters, IPRS, telcos, courts, Auditor-General, international observers) around IEMS |
| L2 — Container | `a2-service-fabric.svg` | 18 deployable K8s services + kit-side APK + CDN; data stores; Kafka bus |
| L2 — Sites | `b1-sites-deployment.svg` | Primary DC + Mirror DC + 5 edge nodes + Escrow; replication flows |
| L2 — Network | `b2-network-path.svg` | Kit → carrier VPN → edge → DC; DDoS scrubbing; SMS/LoRa fallback |
| L2 — Environments | `b3-environment-ladder.svg` | Production → Mirror → Escrow → Staging → Sandbox → Dev → CI |
| L2 — Physical | `b4-rack-ha.svg` | Rack + HA topology (primary site; mirror is identical build) |
| L2 — Field resilience | `b5-field-resilience.svg` | Six-tier field fallback: LTE → 3G → EDGE → SMS → LoRa mesh → courier |
| L2 — Caching | `b6-caching-topology.svg` | Five caching layers with signed invalidation events |
| L2 — Latency | `b7-latency-budget.svg` | End-to-end p99 latency budget for BVR, BVI, and RTS flows |
| L3 — Registration | `c1-registration.svg` | Enrolment → ABIS → Register → Merkle log |
| L3 — Polling day | `c2-polling-day.svg` | BVI identification → Form 34A → ICR → tally → publication → citizen verification |
| L3 — Service mesh | `c3-service-mesh.svg` | SPIFFE SVIDs; Linkerd AuthorizationPolicies; namespace isolation |
| L4 — rts-svc internals | `d1-rts-internals.svg` | Per-constituency shard; ICR pipeline; Merkle builder; Form 34B aggregation |

All SVG + interactive HTML sources in `../diagrams/`. Diagrams are rebuilt from `../diagrams/src/*.mermaid` sources as part of the CI pipeline; any change to service topology must update the relevant Mermaid source.

---

## 21. Document Control

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0 | 2026-08-18 | Datacraft Solutions Architecture | Initial issue — 12-module scope |
| 2.0 | 2026-08-21 | Datacraft Solutions Architecture | Extended to 28 modules; service topology 14 services; caching, resilience, environment sections added |
| 3.0 | 2026-08-21 | Datacraft Solutions Architecture | Full 69-module scope; 18-service topology with grouping rationale; service dependency graph §4; multi-tenancy model §5; service mesh topology §6; failure isolation boundaries §7; full data flows §8; Kafka topic taxonomy §9; API gateway design §10; read/write separation §11; caching topology §12; sync/async patterns §13; security architecture expanded §14; observability SLOs §15; resilience patterns §16; technology rationale expanded §18; C4 model references updated §20 |

Subsequent revisions post-award tracked under Configuration Control Board (P07); every change HSM-signed and appended to the public audit trail (P05 + M15).
