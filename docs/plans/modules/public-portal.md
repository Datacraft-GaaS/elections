# M11 public-portal

| field | value |
|---|---|
| M-number | M11 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-3.9 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M11-public-portal.md |

## Purpose

The statutory public results portal: serve signed constituency/regional/national snapshots, inclusion proofs, and civic information to up to 5 million concurrent readers during result announcement; operate as the primary interface for parties, observers, media, and citizens to independently verify published figures.

## Surfaces and languages

| surface | technology | path under services/portal-svc/ |
|---|---|---|
| Public results portal | Astro on packages/eel-web-astro | astro/public-portal/ |
| Inclusion proof API | Rust (axum + utoipa) | crates/portal-api/ |
| Snapshot cache layer | Valkey (read-through) | platform/valkey/ |
| Admin / content CMS | Django on packages/eel-web-py | django/portal-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PublishedForm | pub_id, form_id, form_type, scope, scope_code, position_code, snapshot_ref, merkle_root, sig, published_at | D | T1-consensus |
| InclusionProof | proof_id, form_id, leaf_index, siblings, root | D | T1-consensus |
| MerkleRoot | root_id, tree_ref, leaf_count, published_at, sig | D | T1-consensus |
| CivicContent | item_id, type, title_en, title_sw, body_html, published_at, expires_at | D | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/portal/public.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /portal/v1/results | 1 signed-static | public | n/a | Latest national snapshot; CDN-cacheable 30 s |
| GET /portal/v1/results/{scope}/{code} | 1 signed-static | public | n/a | Constituency/regional snapshot |
| GET /portal/v1/forms/{form_id}/proof | 2 live-per-user | public | n/a | Inclusion proof for a specific form |
| GET /portal/v1/merkle/{root_id} | 1 signed-static | public | n/a | Merkle root + timestamp; CDN-cacheable |
| GET /portal/v1/turnout | 1 signed-static | public | n/a | Live turnout aggregated from M06; 60 s delay |
| GET /portal/v1/civic/{item_id} | 1 signed-static | public | n/a | Civic content (polling centre list, candidate bios) |
| POST /portal/v1/admin/content | 3 accredited | JWT+SpiceDB | yes | CMS: publish civic content |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.portal.snapshot_served.v1 | emits | eel.portal.audit.v1 | analytics | yes | at-least-once idempotent |
| eel.rts.snapshot_published.v1 | consumes | eel.rts.snapshot.v1 | event-source | yes | exactly-once |
| eel.bvi.turnout_rollup_updated.v1 | consumes | eel.bvi.turnout.v1 | analytics | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end (M10) | internal | integrations/rts-backend/ | yes | snapshot cache; stale reads clearly labelled |
| BVI back-end (M06) | internal | integrations/bvi-backend/ | yes | turnout delayed 60 s; graceful stale |
| Transparency log (P05) | internal | integrations/tlog/ | yes | inclusion proof lookup queued |
| Valkey (platform) | internal | platform/valkey/ | yes | read-through; eviction: publish new on miss |

## ML and AI

none

## Dependencies

- Modules: M10 (snapshot source), M06 (turnout feed), M13 (verifier app reads this portal)
- Platform: P05 (transparency log for proof lookups), Valkey (snapshot cache)
- Packages: eel-crypto-rs, eel-web-astro, eel-web-py

## Feature checklist

1. FR-M11-001: serve latest national/regional/constituency results snapshots; CDN-cacheable with 30 s TTL.
2. FR-M11-002: inclusion proof endpoint: given form_id, return Merkle proof verifiable client-side.
3. FR-M11-003: live turnout display with configurable delay (default 60 s) from BVI feed.
4. FR-M11-004: historical snapshots accessible by version (append-only; no overwrites).
5. FR-M11-005: signed snapshot download: JSON + detached Ed25519 signature file.
6. FR-M11-006: civic content CMS: polling centre locations, candidate list, how-to-verify guide.
7. FR-M11-007: multilingual: English + Kiswahili; extensible to 11 languages.
8. FR-M11-008: WCAG 2.2 AA; screen-reader and keyboard-only support.
9. FR-M11-009: 5M concurrent readers; p95 <= 200 ms; p99 <= 500 ms from platform CDN edge.
10. FR-M11-010: no PII; no biometrics; no voter-specific data in public surface.
11. FR-M11-011: petition-window peak: results remain readable during high-traffic petition filing.
12. FR-M11-012: cache-key includes Merkle root + signature version; no stale poisoning.
13. FR-M11-013: all admin content mutations signed + audit-logged.
14. FR-M11-014: graceful degradation: stale snapshot clearly labelled with last-updated timestamp.
15. FR-M11-015: machine-readable CSV/JSON bulk export with inclusion proofs (tier 4 accredited).

## Acceptance criteria

AC-M11-01: 5M concurrent reads; p95 <= 200 ms; p99 <= 500 ms (load test).
AC-M11-02: inclusion proof verified independently by reference client (integration test).
AC-M11-03: stale snapshot clearly labelled; no silent serving of outdated data (chaos exercise).
AC-M11-04: signed snapshot download integrity verified post-download (UAT).
AC-M11-05: cache poisoning attempt blocked; key includes Merkle root (red-team).
AC-M11-06: WCAG 2.2 AA audit pass (independent audit).
AC-M11-07: civic content published within 30 s of CMS approval (integration test).
AC-M11-08: 99.9% availability during election cycle (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-02: mobile-first; CDN-cached; 3G SLAs.
EC-CON-05: flapping link; read-through Valkey; no error.

**time**
EC-TIM-05: snapshot staleness; last-updated label; stale threshold alert.

**identity**
EC-IDN-02: JWT expires on CMS edit; silent refresh.
EC-IDN-10: SpiceDB down; admin writes blocked; public reads from cache.

**data**
EC-DAT-01: duplicate snapshot push; idempotent upsert on root_id.
EC-DAT-09: Unicode candidate names; NFC; grapheme-safe.
EC-DAT-13: form marked disputed after publication; status shown; proof still valid.

**security**
EC-SEC-06: SSRF via civic content URL fields; allowlist only.
EC-SEC-09: rate-limit on proof endpoint; key by IP+/64; priority shed.
EC-SEC-12: Valkey cache poisoning; ETags include Merkle root.
EC-SEC-16: secrets in logs refused.

**scale**
EC-SCL-01: CDN cache stampede on snapshot publish; staggered TTL jitter.
EC-SCL-04: Valkey eviction at 5M peak; LRU + PG durable store.
EC-SCL-07: 5M concurrent; p95 200 ms; p99 500 ms SLA.

**legal**
EC-TIM-08: petition window; results remain immutable; inclusion proof unaffected.

**human**
EC-HUM-06: screen readers; WCAG 2.2 AA.
EC-HUM-08: 11 languages; no truncation.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | cache key builder, inclusion proof serialiser, stale-label logic, CMS sign/audit | nextest | M11-UNIT- |
| property | inclusion proof always valid for any ingested snapshot; stale label fires on threshold | proptest | M11-PROP- |
| contract | M10 snapshot consumer; M06 turnout consumer; M13 proof endpoint provider | Pact | M11-PACT- |
| integration | Valkey read-through; M10 double; tlog proof lookup | nextest + testcontainers | M11-INT- |
| e2e | snapshot published -> portal visible in < 60 s; proof verifiable in browser | Playwright | M11-E2E- |
| fuzz | snapshot JSON; inclusion proof request | cargo-fuzz | M11-FUZZ- |
| load | 5M concurrent; CDN edge; cache stampede; proof endpoint | k6 | M11-LOAD- |
| chaos | Valkey kill; M10 disconnect; CDN edge failure | Chaos Mesh | M11-CHAOS- |
| security | cache poisoning; OWASP ZAP; Semgrep; stale-injection test | ZAP + Semgrep | M11-SEC- |
| a11y | WCAG 2.2 AA; screen-reader; keyboard-only | axe-core + manual | M11-A11Y- |
| i18n | EN + SW; 11 locales; RTL pseudo | i18n suite | M11-I18N- |

## Runbooks and dashboards

- docs/runbooks/portal-svc-degradation.md (create): Valkey eviction storm; M10 disconnect; CDN purge procedure.
- platform/observability/portal-svc/grafana-public-portal.json: request rate, cache hit rate, inclusion proof latency, stale-label count.
- platform/observability/portal-svc/slo-alerts.yaml: 99.9% burn rate; p99 read latency > 500 ms; stale snapshot > 60 s.

## Open questions

- CDN provider: the platform CDN edge is to be specified by IEBC IT; design assumes generic HTTP/2 CDN; specific configuration TBD.
- Petition-window traffic model: 5M concurrent is the current peak estimate; actual petition-window surge model needs review with IEBC before G3 capacity planning.
