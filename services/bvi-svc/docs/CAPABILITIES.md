# bvi-svc capabilities

Biometric Voter Identification backend: distributes register extracts to 65,000 KIEMS kits, ingests 42 million polling-day identification events across 64 Kafka partitions, revokes kit certificates within 60 seconds, and serves a public turnout portal.

**Tier**: T1-election-critical.
**SLO**: 99.99% availability. BVI 1:1 verification p95 < 800ms, p99 < 1.5s at 18,000 req/s; kit config bundle p99 < 100ms; certificate revocation propagation < 60s; 42M identification events per day.
**RTO**: 2 hours. **RPO**: 15 min.

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
| M06 | BVI backend | WP-3.2 |
| M07 | BVV public verification portal and kiosk | WP-3.3 |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-6 months | Kit config bundle pipeline tested; certificate CA established |
| T-3 months | Register extract distribution tested at scale (65k kits) |
| T-1 day | bvr.register.frozen received; register extract generation immutable |
| Election day T-2 hours | Config bundles pre-positioned to kits; certificate revocation list pushed |
| Election day T+0 | Poll open; identification event ingest starts; turnout rollups begin |
| Election day T+10 hours | Poll close; final turnout rollup published; BVV portal shows final figures |
| T+3 days | Post-election: objection window closes; identification event data frozen |

---

## 2. End-to-end pipelines

### 2.1 Kit config bundle distribution (pre-poll)

```
bvr.register.frozen event received by bvi-api
  -> generate register extract (voter_id, polling_station_id, biometric_template_hash) per station
  -> store in kit_configs (station_code, version, bundle_id, voter_list_hash, geofence_hash, cert_hash)
  -> sign bundle (transparency-svc hsm-signer Ed25519)
  -> upload signed bundle to MinIO (minio_key in kit_configs)
  -> kit GET /api/v1/bvi/kits/{serial}/config-bundle -> download + verify signature
  -> kit caches bundle in local Isar + SQLCipher encrypted storage (biometric-edge-svc)
```

### 2.2 Polling-day identification event ingest

```
kit (local 1:1 biometric match successful)
  -> POST /api/v1/bvi/kits/{serial}/events (batch, signed with kit SPIRE SVID)
  -> bvi-api: verify SPIRE SVID certificate not revoked (kit_certs.revoked check)
  -> check honey_mode: if kit_certs.revoked=true -> route to honey-mode handler
     (accept event with honey_mode=true; alert silently; do not enrich kit with suspicion)
  -> idempotency: UNIQUE (event_id) on identification_events table
  -> no-double-ID check: UNIQUE (voter_id, election_id) WHERE outcome='matched'
  -> insert into identification_events (event_id, voter_id, station_code, kit_serial,
     modality, outcome, ts_hlc, gps_fix, kit_sig, honey_mode)
  -> Kafka produce: eel-bvi-identification (64 partitions, key = station_code)
  -> bvi-turnout worker: consume from Kafka -> REFRESH MATERIALIZED VIEW mv_turnout_station
     -> update turnout_rollups -> publish bvi.turnout.updated CloudEvent
```

### 2.3 Certificate revocation (< 60s SLA)

```
Commission operator: POST /api/v1/bvi/kits/{serial}/revoke (admin role)
  -> bvi-api: set kit_certs.revoked=true, revoked_at, revoked_reason, revoked_by
  -> publish bvi.kit.cert.revoked CloudEvent (Kafka: eel-bvi-kit-mgmt)
  -> biometric-edge-svc consumes event: pushes updated CRL to kit on next sync
  -> timing: Kafka pub + edge-svc push < 60s end-to-end
  -> subsequent kit API calls: honey-mode routing (events ingested but flagged)
```

### 2.4 Turnout rollups (bvi-turnout worker)

```
Kafka consumer (eel-bvi-identification, 64 partitions)
  -> exactly-once via Kafka transactions (KIP-98)
  -> on batch commit: REFRESH MATERIALIZED VIEW mv_turnout_station
  -> update turnout_rollups (scope, scope_code, authenticated_count, registered_count)
  -> aggregate to constituency, county, national scope
  -> publish bvi.turnout.updated.v1 CloudEvent hourly
```

### 2.5 Citizen BVV lookup (M07)

```
citizen GET /api/v1/bvv/voters/{id}/verification (public, no auth)
  -> bvv Astro portal or bvi-api
  -> timing-constant lookup (constant-time compare of id hash; no timing oracle)
  -> enumeration cap: rate-limit 10 req/min per /64 subnet; return 429 on excess
  -> return { identified: bool, station_code, verified_at } (no PII)
```

---

## 3. Domain aggregates and state stores

Tables from `migrations/V1__init.sql`, `V2__kit_configs.sql`, `V3__identification_events.sql`, `V4__turnout_rollups.sql`, `V5__kit_certs.sql`, `V6__objections.sql`, `V7__voter_lookup_audit.sql`, `V8__outbox.sql`.

| Table | Purpose |
|-------|---------|
| `kit_configs` | Per-station config bundles. Columns: `id`, `station_code`, `version`, `bundle_id`, `voter_list_hash`, `geofence_hash`, `cert_hash`, `minio_key`, `signed_at`, `created_at`. Unique on `(station_code, version)`. |
| `identification_events` | Voter identification event mirror (Kafka is source of truth; PG for reads). Columns: `id`, `event_id` (UNIQUE for idempotency), `voter_id`, `station_code`, `kit_serial`, `modality` (fingerprint/iris/face/nfc), `outcome` (matched/no_match/timeout/error), `ts_hlc`, `gps_fix` (JSONB), `kit_sig`, `received_at`, `honey_mode`, `election_id`. Unique on `(voter_id, election_id) WHERE outcome='matched'` (no-double-ID constraint). |
| `turnout_rollups` | Aggregate turnout by scope. Columns: `id`, `scope` (station/constituency/county/national), `scope_code`, `authenticated_count`, `registered_count`, `updated_at`. Unique on `(scope, scope_code)`. |
| `mv_turnout_station` | Materialised view: per-station identification counts from `identification_events WHERE outcome='matched'`. Unique index on `scope_code`. |
| `kit_certs` | Kit certificate registry. Columns: `id`, `kit_serial` (UNIQUE), `cert_pem`, `issued_at`, `revoked`, `revoked_at`, `revoked_reason`, `revoked_by`. Partial index on `revoked=true`. |
| `audit.shadow_log` | Audit shadow for all table mutations. Columns: `id`, `table_name`, `operation`, `old_row` (JSONB), `new_row` (JSONB), `changed_at`. |

V6 `objections` and V7 `voter_lookup_audit` schemas were referenced in migration filenames but content not read; columns are planned. V8 `outbox` table exists for transactional CloudEvent relay.

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| Signed per-station config bundle | `src/` (bvi-api) | Ed25519 over BLAKE3(bundle JSON) via transparency-svc hsm-signer | Bernstein et al., EUROCRYPT 2011; O'Connor et al., BLAKE3 2020 | ISO/IEC 19790 |
| Identification event ingest (64 partitions) | `src/` (bvi-api) | Kafka producer key = hash(station_code) % 64; exactly-once via Kafka transactions | Apache Kafka KIP-98, Narkhede et al. 2017 | none binding |
| Turnout materialised view rollup | `migrations/V4__turnout_rollups.sql` | REFRESH MATERIALIZED VIEW CONCURRENTLY on `mv_turnout_station` | PostgreSQL 17 Materialised Views, postgresql.org 2024 | none binding |
| No-double-ID constraint | `migrations/V3__identification_events.sql` | Partial unique index on `(voter_id, election_id) WHERE outcome='matched'` | database-level uniqueness; Kenya Elections Act 2011 §39 | Kenya Elections Act 2011 §39 |
| Honey-mode routing for revoked kits | `src/` (bvi-api) | On revoked kit: accept events with `honey_mode=true`; alert silently; do not signal kit | "Honeypot", Spitzner, "Honeypots: Tracking Hackers", Addison-Wesley 2002 | none binding |
| Timing-constant BVV lookup | `src/` (bvi-api or bvv) | Constant-time hash comparison of voter_id to prevent timing oracle; rate-limit 10/min per /64 | Bernstein, "Timing Attacks on Implementations of Diffie-Hellman, RSA, DSS", 1998 | none binding |
| Enumeration cap | `src/` (bvi-api) | Token bucket rate limiter per /64 subnet (Leaky Bucket: Tanenbaum, "Computer Networks", 2003) | Tanenbaum, "Computer Networks" 5th ed., Pearson 2010 | none binding |
| Kit certificate extension timing | `crates/` (planned) | Certificate validity extension on same-day application per Kenya Elections Act §14A; 60s revocation SLA | Kenya Elections Act 2011 §14A | Kenya Elections Act 2011 §14A |

---

## 5. Crypto and trust anchoring

Kit certificates are issued by a certificate authority established in biometric-edge-svc (`crates/kit-attest`, SPIRE-issued SVIDs). bvi-svc maintains the `kit_certs` table as the revocation registry. On revocation, `kit_certs.revoked=true` is set and a CloudEvent published. biometric-edge-svc pushes the updated CRL to the kit on next heartbeat or sync.

Config bundles are signed with transparency-svc's HSM Ed25519 key (key role `sth`). Kits verify the bundle signature before accepting configuration. This prevents a compromised bvi-svc instance from distributing altered voter lists.

Identification events carry `kit_sig` (Ed25519 signature by the kit's SPIRE SVID private key) over the event payload. bvi-api verifies the signature against the SPIRE trust bundle before accepting the event.

---

## 6. Event surface

### Emitted (via outbox, Kafka)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.bvi.identification.recorded.v1` | eel-bvi-identification | `event_id`, `voter_id`, `station_code`, `kit_serial`, `modality`, `outcome`, `ts_hlc` |
| `eel.bvi.turnout.updated.v1` | eel-bvi-turnout | `scope`, `scope_code`, `authenticated_count`, `registered_count`, `updated_at` |
| `eel.bvi.kit.cert.revoked.v1` | eel-bvi-kit-mgmt | `kit_serial`, `cert_hash`, `revoked_at`, `reason` |
| `eel.bvi.register.frozen.v1` | eel-bvi-register | `election_id`, `extract_version`, `frozen_at` |

All CloudEvents 1.0. Source: `spiffe://eel.local/svc/bvi-svc`.

### Consumed

| Event type | Topic | Action |
|------------|-------|--------|
| `eel.bvr.register.frozen.v1` | eel-bvr-events | Trigger register extract generation; distribute to kits |

---

## 7. API surface

### REST (bvi-api, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/bvi/kits/{serial}/config-bundle` | Download register extract, config, and CRL for kit (SPIRE SVID required) |
| POST | `/api/v1/bvi/kits/{serial}/events` | Kit submits batch identification events |
| GET | `/api/v1/bvi/turnout/{station_id}` | Turnout aggregate for station (live during polling) |
| GET | `/api/v1/bvv/voters/{id}/verification` | Citizen verifies identification status (public, no auth) |
| POST | `/api/v1/bvi/kits/{serial}/revoke` | Revoke kit certificate (Commission admin role) |
| GET | `/api/v1/bvi/health` | Liveness; 503 until register extract cached |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### Elixir LiveView (ex/ directory, observer UI)

Real-time turnout dashboard for election observers. Subscribes to `eel-bvi-turnout` Kafka topic via Phoenix PubSub; renders live station/constituency turnout maps. No write surface.

### bvv Astro portal (astro/, port 3000, public)

Static SSR portal. Serves citizen-facing voter identification status and turnout visualization. Data sourced from `turnout_rollups` and `identification_events` via read-only PG replica.

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| bvr-svc (register freeze) | Kafka consumer in bvi-api | Topic `eel-bvr-events` |
| transparency-svc (bundle signing) | gRPC client | `hsm-signer.Sign` RPC |
| biometric-edge-svc (CRL push) | Kafka producer `eel-bvi-kit-mgmt` | Event consumed by edge-svc |
| MinIO (bundle storage) | S3-compatible client in bvi-api | `PUT/GET` bundle objects |
| SPIRE (kit SVID verification) | SPIFFE workload API trust bundle in bvi-api | `spiffe://eel.local/kit/{serial}` SVID validation |

---

## 9. Sharding, indexing, and scale

- `identification_events` ingestion: 64 Kafka partitions keyed by `station_code` hash. 42M events/day = 486 events/s average; burst at polls-open 18,000/s (6 hours = 7.8M events in first hour).
- `identification_events` PG indexes: `station_code`, `kit_serial`, `received_at DESC`. Partial unique on `(voter_id, election_id) WHERE outcome='matched'`.
- `mv_turnout_station` materialised view refreshed concurrently every 5 minutes during polling day; unique index on `scope_code` enables fast refresh.
- `turnout_rollups` unique on `(scope, scope_code)`; 46,229 station rows + 290 constituency + 47 county + 1 national = 46,567 rows; full table fits in PG shared_buffers.
- `kit_certs` has at most 65,000 rows; certificate lookup is O(1) via UNIQUE index on `kit_serial`.
- Scale: 65,000 simultaneous kit config-bundle downloads at poll-open requires MinIO read parallelism; bundles are pre-staged 2 hours before poll-open.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | Event ingest, turnout rollup, and BVV portal at full capacity | docs/runbooks/bvi-svc.md |
| Degraded (bvr-svc frozen event missing) | Network partition | Config bundle generation delayed; kits fall back to cached register (biometric-edge-svc 7-day offline queue) | docs/runbooks/bvi-svc.md#frozen-delay |
| Degraded (MinIO unavailable) | MinIO service down | Config bundle download fails; kits use cached copy; alert fired | docs/runbooks/bvi-svc.md#minio-unavailable |
| Degraded (Kafka unavailable) | Kafka partition loss | Identification events accepted to PG outbox; turnout rollup stalled; alert fired | docs/runbooks/bvi-svc.md#kafka-recovery |
| Honey-mode (revoked kit) | kit_certs.revoked=true | Events accepted but flagged `honey_mode=true`; operator alerted; kit not informed | docs/runbooks/bvi-svc.md#revoked-kit |
| Read-only (DB primary down) | PG primary failover | Event reads from replica; new event writes to Kafka outbox; RTO 2 hours | docs/runbooks/bvi-svc.md#db-failover |

---

## 11. Data lifecycle and retention

| Table | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `identification_events` | Class A: election-critical | Permanent (legal audit record; Kenya Elections Act) |
| `turnout_rollups` | Class A | Permanent |
| `mv_turnout_station` | Class C: derived | Refreshable; retain 90 days |
| `kit_configs` | Class B: operational | 10 years |
| `kit_certs` | Class B | 10 years (revocation records permanent) |
| `audit.shadow_log` | Class A | Permanent |
| Outbox | Class C: ephemeral | 30 days |

No voter PII is stored in bvi-svc. `voter_id` is an opaque UUID7; PII is decrypted only by bvr-svc on authorized lookup.

---

## 12. Observability

### Dashboards (Grafana, d/bvi)

- BVI overview: request rate, p95/p99 latency, identification rate by station.
- Turnout tracking: hourly rollup by station/constituency, live turnout percentage.
- Kit health: config bundle fetch rate, event upload latency, CRL propagation time.
- Honey-mode tracker: count of revoked-kit events flagged.

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `bvi_identification_rate_anomaly` | Turnout % deviates > 10% from projection | warning |
| `bvi_kit_event_latency_high` | Event ingest p99 > 1.5s | critical |
| `bvi_cert_revocation_propagation_slow` | Revocation event delay > 60s | warning |
| `bvi_honey_mode_events` | Any honey_mode=true events received | warning |
| `bvi_double_id_attempt` | Unique constraint violation on `(voter_id, election_id)` | critical |

### Key metrics

- `eel_bvi_identification_latency_seconds` - Histogram: event ingestion time.
- `eel_bvi_turnout_by_station` - Gauge: voter count by station (updated every 5 min).
- `eel_bvi_kit_config_age_seconds` - Gauge: kit config bundle staleness.
- `eel_bvi_revoked_kit_events_total` - Counter: honey-mode events.
- `eel_bvi_cert_revocation_lag_seconds` - Gauge: revocation propagation time.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | `cargo nextest run --lib` | Idempotency check, honey-mode routing, no-double-ID constraint logic, timing-constant comparison |
| Property | proptest | Event ingest: any batch of N events for the same voter_id produces exactly 1 matched outcome |
| Contract | OpenAPI conformance | REST response shapes; CloudEvent envelope schema |
| Integration | `cargo nextest run --test '*' -- --test-threads=1` | PG + Kafka testcontainers; certificate revocation pipeline; turnout materialised view refresh |
| Load | k6 (`loadtest/bvi-identify-18k.js`) | 18,000 req/s identification event ingest; p99 < 1.5s |
| Chaos | planned (Wave 3) | MinIO unavailable; Kafka partition loss; revoked kit attempting events |
| Security | timing attack test (planned) | BVV lookup timing variance < 1ms across 1000 samples |

---

## 14. Follow-ups and known limitations

- V6 `objections` and V7 `voter_lookup_audit` table DDL exists as migration filenames but content was not fully read. Their columns are not enumerated here. Verify and document in Wave 2.
- `crates/` directory in bvi-svc is noted in README as "empty; crates aspirational". All business logic is in `src/` binary entry points. Refactoring into crates is planned for Wave 2.
- Certificate issuance flow for `kit_certs` is not fully documented in bvi-svc. The CA is held by biometric-edge-svc (`crates/kit-attest`); bvi-svc only holds the revocation registry. Cross-service certificate lifecycle needs a clear runbook.
- Elixir LiveView observer dashboard (`ex/` directory) is confirmed present but its Kafka consumer implementation was not verified. Treat as planned until integration tested.
- Kenya Elections Act §14A extension timing: the 60s revocation SLA is a design target; whether §14A requires a specific legal notice period before a certificate can be revoked needs legal review.

---

## Appendix A: Identification event no-double-ID enforcement

The no-double-ID constraint prevents a voter from being identified (marked as `outcome='matched'`) more than once per election. This is enforced at multiple levels:

1. **Database**: Partial unique index `identification_events_voter_election_uq ON identification_events(voter_id, election_id) WHERE outcome='matched'`. Any duplicate successful identification attempt produces a PostgreSQL unique violation, which the API layer converts to a 409 Conflict response.
2. **Kafka consumer**: The bvi-turnout worker uses Kafka transactions for exactly-once semantics. If a duplicate event arrives (same event_id), the UNIQUE constraint on `event_id` prevents double-insertion at the PG layer.
3. **Kit-side**: bvi-kit records a local flag in Isar after a successful identification; any subsequent attempt for the same voter on the same kit is rejected before transmission. This prevents both accidental repeats and deliberate re-scanning.

The `election_id` column (added via `ALTER TABLE` in `V3__identification_events.sql`) defaults to `'EL-2027'` for the current election cycle. When multi-election support is needed, a migration changes the default.

---

## Appendix B: Register extract bundle structure

A kit config bundle (stored in MinIO, referenced by `kit_configs.minio_key`) contains:

```json
{
  "bundle_id": "uuid7-...",
  "station_code": "001-001-001",
  "extract_version": 3,
  "voter_list": [
    { "voter_id": "...", "biometric_hash": "...", "name_nfc": "..." }
  ],
  "geofence": { "type": "Polygon", "coordinates": [...] },
  "geofence_hash": "sha256:...",
  "cert_revocation_list": ["cert_thumbprint_1", "cert_thumbprint_2"],
  "cert_hash": "sha256:...",
  "signed_at": "2027-07-15T06:00:00Z",
  "signature": "ed25519:..."
}
```

Key design decisions:
- `voter_list` is filtered to only voters registered at the specific station (polling_station_id match). A station receives only its own voters.
- `biometric_hash` is a hash of the voter's biometric template (not the template itself; raw biometric data never leaves abis-svc).
- The bundle is signed over `BLAKE3(canonical_json(bundle))` by transparency-svc HSM. The kit verifies this signature using the transparency-svc public key pinned at build time.
- Bundle size varies by station: typical station 300-500 voters; maximum (urban stations) ~1,200 voters. Compressed bundle < 200 KB for typical stations.

---

## Appendix C: Turnout projection and anomaly detection

The `bvi_identification_rate_anomaly` alert fires when a station's turnout percentage deviates more than 10% from the pre-election projection.

Projections are loaded at poll-open from historical data:
- Previous election turnout for the same station (adjusted for population change).
- Demographic model (age distribution, mobility patterns, weather).
- Stored in `turnout_rollups.registered_count` (denominator) set at poll-open from the register extract.

The anomaly check compares `authenticated_count / registered_count` against the projected range every 30 minutes. Triggers:
- `< projection - 10%`: possible kit malfunction, voter access issue, or very low turnout. Field supervisor alert.
- `> projection + 10%`: possible duplicate identification, bot traffic, or very high turnout. Security team alert.

The alert is informational during the first 2 hours (polls take time to get going); it becomes warning severity after hour 3 and critical only if the deviation exceeds 25%.

---

## Appendix D: Elixir LiveView observer dashboard

The `ex/` directory contains an Elixir/Phoenix LiveView application providing a real-time observer dashboard. It is a separate process from bvi-api and does not accept writes. Key design:

- Subscribes to Kafka topic `eel-bvi-turnout` via an Elixir Kafka consumer (Franz or Broadway).
- Uses Phoenix PubSub to broadcast turnout updates to connected LiveView clients.
- Renders a choropleth map (Kenya constituencies color-coded by turnout %) with < 5s update lag from when bvi-turnout publishes a rollup.
- Authentication: election observers receive a time-limited JWT signed by the Commission's observer credential service. The JWT is verified by the Phoenix Guardian library.
- No write surface: the observer dashboard is strictly read-only. It cannot modify identification events, turnout rollups, or kit configurations.
- Deployment: co-deployed as a separate container in the same Kubernetes namespace as bvi-api; served on port 4000.

---

## Appendix E: Kit certificate lifecycle

Kit certificates follow this lifecycle from issuance to revocation:

```
biometric-edge-svc kit-attest: issue SPIRE SVID -> bvi-svc kit_certs INSERT
  kit_certs: { kit_serial, cert_pem, issued_at, revoked=false }
  Normal operation: kit uses cert for API auth
  Commission revokes: POST /api/v1/bvi/kits/{serial}/revoke
    -> kit_certs: { revoked=true, revoked_at, revoked_reason, revoked_by }
    -> bvi.kit.cert.revoked CloudEvent published
    -> biometric-edge-svc: pushes updated CRL on next kit heartbeat
    -> kit: CRL received; subsequent bvi-svc calls rejected at gateway
    -> bvi-api: subsequent events from revoked kit routed to honey_mode=true handler
```

Certificate extension (`Kenya Elections Act §14A`) allows a kit certificate to be extended on the day of a polling day extension due to emergency or disruption. The extension process:
1. Commission declares a polling extension (legal instrument).
2. Commission admin POST /api/v1/bvi/kits/{serial}/extend (planned endpoint).
3. kit_certs: `expires_at` updated to new poll-close time.
4. New CRL pushed to all kits to reflect the extension.
5. Extension event appended to transparency log.

Extension endpoint is planned (Wave 2); current implementation does not support same-day certificate extension.
