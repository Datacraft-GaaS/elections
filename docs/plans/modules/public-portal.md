# M11 public-portal

| field | value |
|---|---|
| M-number | M11 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-3.9 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M11-public-portal.md |

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

Results display:
1. FR-M11-001: home dashboard: national totals per elective position and turnout with time-of-last-update. (spec FR-M11-001)
2. FR-M11-002: drill-down navigation: national to county to constituency to ward to polling station. (spec FR-M11-002)
3. FR-M11-003: per-polling-station page: signed Form 34A image, extracted tally per candidate, Merkle-inclusion proof, transparency-log index, PO and agent signatures. (spec FR-M11-003)
4. FR-M11-004: provenance overlay on each form: HSM signature indicator, Merkle-inclusion index, and escrow-mirror confirmation. (spec FR-M11-016)
5. FR-M11-005: live turnout feed with configurable delay (default 60 s) to prevent gaming. (spec FR-M11-017)
6. FR-M11-006: transparency dashboard: every published Merkle root, every quorum operation, and every high-severity incident (anonymised). (spec FR-M11-009)
7. FR-M11-007: historical snapshot access by version; append-only; no overwrites; earliest snapshot preserved.
8. FR-M11-008: disputed form status shown; original signed image and Merkle proof remain accessible to petitioners.

Downloads and exports:
9. FR-M11-009: bulk download of signed forms per constituency, county, and national with Ed25519 signature bundles. (spec FR-M11-004)
10. FR-M11-010: transparency-log root retrieval: any citizen can fetch current root and any historical root. (spec FR-M11-005)
11. FR-M11-011: inclusion-proof retrieval: given form_id or station code, return Merkle-inclusion proof verifiable client-side. (spec FR-M11-006)
12. FR-M11-012: signed snapshot download: JSON plus detached Ed25519 signature file verifiable against the Commission public key. (spec FR-M11-008)
13. FR-M11-013: RSS/Atom feed of published forms for automation-friendly consumers. (spec FR-M11-013)
14. FR-M11-014: OpenAPI JSON endpoints for programmatic access with rate limit. (spec FR-M11-014)
15. FR-M11-015: machine-readable CSV/JSON bulk export with inclusion proofs (tier 4 accredited only).
16. FR-M11-016: petition-window peak: results immutable; inclusion proofs unaffected; bulk downloads throttled; off-peak windows offered.

Architecture and reliability:
17. FR-M11-017: static-first CDN architecture: pages served from CDN cache; origin serves update generator only. (spec FR-M11-007)
18. FR-M11-018: signed URLs on CDN; tampered snapshots detectable on client-side verification. (spec FR-M11-008)
19. FR-M11-019: active-active HA across primary and mirror DCs; CDN edge serves read traffic; failover RTO <= 30 s. (spec NFR-M11-006)
20. FR-M11-020: graceful degradation: stale snapshot clearly labelled with last-updated timestamp; no silent serving of outdated data.
21. FR-M11-021: DDoS resilience: origin protected by upstream scrubbing, anycast, WAF, and L7 challenge; static snapshots continue on origin loss. (spec NFR-M11-003)
22. FR-M11-022: 5M unique visitors in 1 hour peak; 50M total in first 24 h; p95 <= 200 ms; p99 <= 500 ms. (spec NFR-M11-001)
23. FR-M11-023: availability 99.9% during election cycle; degrade to signed CDN static on origin loss. (spec NFR-M11-004)

Security and integrity:
24. FR-M11-024: cache key includes Merkle root and signature version; no stale poisoning on CDN or Valkey. (spec FR-M11-012)
25. FR-M11-025: every response carries Ed25519 signature verifiable against the Commission public key on canonical domains. (spec NFR-M11-005)
26. FR-M11-026: no PII: forms show only statutorily required data; agent identities anonymised. (spec FR-M11-015)
27. FR-M11-027: rate limit on proof and bulk-download endpoints; key by IP and /64; priority shed on burst.
28. FR-M11-028: SSRF prevention on civic CMS URL fields: allowlist only; no private IP ranges.
29. FR-M11-029: all admin CMS mutations signed and audit-logged before publication.
30. FR-M11-030: secrets in logs refused; Semgrep Tier A/B log ban enforced in CI.
31. FR-M11-031: injection prevention on all CMS and API inputs; parameterised queries; ZAP scan clean.

Content and accessibility:
32. FR-M11-032: civic CMS: polling centre locations, candidate list, how-to-verify guide. (spec FR-M11-006)
33. FR-M11-033: multilingual: English and Kiswahili on all user-facing surfaces; extensible to 11 languages. (spec FR-M11-010)
34. FR-M11-034: WCAG 2.2 AA: screen-reader; keyboard-only; text-only fallback for slow connections. (spec FR-M11-011)
35. FR-M11-035: mobile-first responsive design; page weight < 200 KB on initial load over 3G. (spec FR-M11-012)
36. FR-M11-036: KSL video explainer for result verification process; captions on all video.
37. FR-M11-037: high-contrast mode and configurable text scaling on web portal.
38. FR-M11-038: colour-blind accessible status indicators: symbol plus text plus pattern; delta-E registry.

Monitoring:
39. FR-M11-039: Grafana dashboard: CDN cache-hit ratio, snapshot age, Merkle anchor lag, proof request latency, bulk-download queue depth.
40. FR-M11-040: SLO burn-rate alerts: p95/p99 read latency; snapshot staleness threshold; origin availability.

Transparency log:
41. FR-M11-041: cache last N transparency-log roots; degrade gracefully if log unreachable; alert on staleness.
42. FR-M11-042: escrow mirror status shown on transparency dashboard; lag alerted if mirror falls behind.

Legal and statutory:
43. FR-M11-043: petition-window countdown displayed: T+7 days from national declaration per s.75(1).
44. FR-M11-044: presidential run-off rules displayed if no majority declared.
45. FR-M11-045: referendum county-majority threshold visualised: 24-of-47 counties requirement clearly shown.

CMS governance:
46. FR-M11-046: CMS approval workflow: content staged, previewed, and dual-signed before going live.
47. FR-M11-047: content version history retained; rollback available to any previous signed version.
48. FR-M11-048: content update live within 30 s p95 of CMS approval.

API governance:
49. FR-M11-049: OpenAPI published at well-known URL; openapi-diff clean on every release.
50. FR-M11-050: generated API clients compile and smoke test in 4 languages on each publish.

## Acceptance criteria

AC-M11-01: 5M concurrent reads; p95 <= 200 ms; p99 <= 500 ms from CDN edge (load test).
AC-M11-02: inclusion proof verified independently by reference client within 30 s (integration test).
AC-M11-03: stale snapshot clearly labelled; no silent serving of outdated data (chaos exercise).
AC-M11-04: signed snapshot download integrity verified post-download (UAT).
AC-M11-05: CDN cache poisoning attempt blocked; cache key includes Merkle root (red-team).
AC-M11-06: DDoS on origin does not stop portal; static CDN snapshots continue serving (chaos exercise).
AC-M11-07: WCAG 2.2 AA audit pass: axe zero violations; screen-reader traversal completes (independent audit).
AC-M11-08: civic content live within 30 s of CMS approval (integration test).
AC-M11-09: 99.9% availability during election cycle (SLO measurement).
AC-M11-10: no PII or biometric data in any published surface (automated scan plus manual review).
AC-M11-11: 11-language smoke test: no truncated strings; ICU plural forms correct (i18n test).
AC-M11-12: petition-window peak: all inclusion proofs retrievable under 5M concurrent (load test).
AC-M11-13: transparency-log root cached; stale root alerted within 60 s of log failure (chaos exercise).

## Applicable edge cases

connectivity:
EC-CON-02: mobile-first; CDN-cached; 3G SLAs; page weight < 200 KB on initial 3G load.
EC-CON-05: flapping link; read-through Valkey cache; no error surfaced to user.

time:
EC-TIM-05: snapshot staleness; last-updated label; stale-threshold alert fires.
EC-TIM-07: extended tallying window; portal updated when HSM-signed extension event received.
EC-TIM-08: petition window T+7 midnight EAT; results immutable; s.75(1) inclusive boundary enforced.

identity:
EC-IDN-02: JWT expires on CMS admin edit; in-flight completes; next request 401 plus silent refresh.
EC-IDN-10: SpiceDB down; admin CMS writes blocked; public reads served from CDN cache.
EC-IDN-11: stale JWKS on CMS admin path; refetch on kid miss; no 401 storm.

data:
EC-DAT-01: duplicate snapshot push same root_id; idempotent upsert; no duplicate publication.
EC-DAT-02: same snapshot key different body; 422 idempotency-key-reuse; alert.
EC-DAT-09: Unicode candidate names in snapshot JSON; NFC normalisation; grapheme-safe.
EC-DAT-13: form marked disputed after publication; disputed status shown; proof still valid.

security:
EC-SEC-05: injection on CMS admin inputs and API query params; parameterised only; Semgrep clean.
EC-SEC-06: SSRF via civic CMS URL fields; allowlist; no private IP ranges.
EC-SEC-09: rate-limit evasion on proof and bulk-download endpoints; key by IP+/64; priority shed.
EC-SEC-12: Valkey cache poisoning; cache keys and ETags include Merkle root and sig version.
EC-SEC-16: secrets in logs refused; no the Commission signing keys in telemetry.

scale:
EC-SCL-01: CDN cache stampede on snapshot publish; staggered TTL jitter; origin p99 <= 500 ms.
EC-SCL-02: thundering herd on STH root fetch; SDK jitter; ETag 304; rate-limit.
EC-SCL-04: Valkey eviction at 5M peak; PG-durable store; Valkey is cache only; no idempotency loss.
EC-SCL-07: 5M concurrent; p95 200 ms; p99 500 ms SLA sustained 30 min.
EC-SCL-08: Valkey eviction of snapshot cache key; PG durable store serves; Valkey refills async.

legal:
EC-LEG-04: recount ordered; new snapshot version via compensating event; both versions shown with proofs.
EC-LEG-05: presidential run-off; rules engine blocks certificate; run-off portal page created.
EC-LEG-07: referendum 24-of-47 county threshold; exact >= semantics enforced; visualisation accurate.

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

- CDN provider: the platform CDN edge is to be specified by the Commission IT; design assumes generic HTTP/2 CDN; specific configuration TBD.
- Petition-window traffic model: 5M concurrent is the current peak estimate; actual petition-window surge model needs review with the Commission before G3 capacity planning.
