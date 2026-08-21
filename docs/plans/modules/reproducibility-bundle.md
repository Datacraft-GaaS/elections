# M14 reproducibility-bundle

| field | value |
|---|---|
| M-number | M14 |
| Home service | transparency-svc (ADR-0020) |
| Owning work package | WP-4.6 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M14-reproducibility-bundle.md |

## Purpose

Assemble and publish the statutory post-election reproducibility package: a self-contained, independently verifiable archive of all signed forms, participation tokens, Merkle proofs, source code, and build manifests, enabling any technically capable citizen, journalist, or party to independently re-derive and verify every published result figure.

## Surfaces and languages

| surface | technology | path under services/transparency-svc/ |
|---|---|---|
| Bundle CLI (offline use) | Rust (clap) | crates/bundle-cli/ |
| Bundle web portal | Astro on packages/eel-web-astro | astro/reproducibility-portal/ |
| Bundle API | Rust (axum + utoipa) | crates/transparency-api/ |
| Bundle assembly batch | Rust | crates/bundle-batch/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Bundle | bundle_id, election_id, components_manifest, merkle_root, assembled_at, sig, size_bytes | D | T1-audit |
| BundleComponent | comp_id, bundle_id, type (forms/tokens/src/build), ref, hash, size_bytes | D | T1-audit |
| ParticipationToken | token_sha256, station_code, issued_at, inclusion_proof_ref | D | T1-audit |
| BugBounty | bounty_id, reporter_hash, finding_summary, status, reward_level | C | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/transparency/bundle.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /transparency/v1/bundles | 1 signed-static | public | n/a | List available bundles for an election |
| GET /transparency/v1/bundles/{id} | 1 signed-static | public | n/a | Bundle manifest + component list |
| GET /transparency/v1/bundles/{id}/download | 4 bulk | public | n/a | Signed bundle archive download |
| GET /transparency/v1/tokens/{sha}/proof | 2 live-per-user | public | n/a | Participation token inclusion proof |
| POST /transparency/v1/bug-bounty | 2 live-per-user | public | yes | Submit bug bounty / reproducibility finding |
| GET /transparency/v1/bug-bounty/{id} | 3 accredited | JWT+SpiceDB | n/a | Triage + status (IEBC internal) |
| POST /transparency/v1/bundles/assemble | 3 accredited | JWT+SpiceDB | yes | Trigger bundle assembly (post-election) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.transparency.bundle_assembled.v1 | emits | eel.transparency.bundle.v1 | operational | yes (HSM) | exactly-once |
| eel.transparency.bundle_published.v1 | emits | eel.transparency.bundle.v1 | operational | yes | exactly-once |
| eel.transparency.bug_bounty_submitted.v1 | emits | eel.transparency.bounty.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form_ingested.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.merkle_root_anchored.v1 | consumes | eel.rts.merkle.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | form + token pull; retry |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on bundle sign |
| Transparency log (P05) | internal | integrations/tlog/ | yes | Merkle anchoring queued |
| KBC SFTP | external | integrations/kbc-sftp/ | yes | statutory broadcast mirror |
| Gazette (Gov Printer) | external | integrations/gazette/ | yes | notice publication |
| MinIO (platform) | internal | platform/minio/ | yes | large archive blob store |

## ML and AI

none

## Dependencies

- Modules: M08 (participation token source), M10 (form + Merkle source), M13 (reads token proof endpoint), M21 (RLA reads from bundle)
- Platform: P05 (audit), P06 (HSM), MinIO (blob storage)
- Packages: eel-crypto-rs, eel-web-astro

## Feature checklist

1. FR-M14-001: assemble post-election reproducibility bundle containing all signed Form 34A/B/C images, participation tokens, Merkle proofs, source code snapshot, build manifests, and SBOM.
2. FR-M14-002: deterministic bundle assembly: identical inputs produce byte-identical archive across runs and CPU architectures (x86_64, aarch64).
3. FR-M14-003: HSM-signed bundle manifest (Ed25519) with SHA-256 hash of every included component; manifest verifiable offline with standard tooling.
4. FR-M14-004: compressed archive format: tar.zst with integrity header; extraction validates before any file write.
5. FR-M14-005: public download endpoint: signed CDN URL; content-addressed; immutable once published.
6. FR-M14-006: CLI verifier (`eel-repro verify bundle.tar.zst`): outputs PASS/FAIL and diff of any diverging tallies; no network required.
7. FR-M14-007: participation token inclusion proof API: any citizen queries their token SHA-256 and receives a Merkle path proving inclusion.
8. FR-M14-008: web portal: drag-and-drop bundle verification; all verification logic runs in-browser (WASM); zero upload of data to server.
9. FR-M14-009: REST API: programmatic bundle list, manifest fetch, download URL, token proof, bug-bounty submission.
10. FR-M14-010: batch pipeline: triggered by `eel.rts.merkle_root_anchored.v1`; assembles national and constituency bundles; emits `eel.transparency.bundle_assembled.v1`.
11. FR-M14-011: statutory mirroring: bundle pushed to KBC SFTP and Government Gazette API within 1 h of publication; retry queue with alert at 30 min.
12. FR-M14-012: bug bounty intake: public POST endpoint; IEBC internal triage workflow; status visible to submitter via anonymous token.
13. FR-M14-013: versioned bundle format (semver): forward-compatible; correction bundles carry a delta manifest; prior versions remain immutable.
14. FR-M14-014: constituency-level partial bundle: download subset scoped to a single constituency; shares same manifest format.
15. FR-M14-015: SBOM embedded in bundle root; every build artifact traceable to source commit and dependency hash.
16. FR-M14-016: air-gapped verification guide: printed PDF + digital Markdown bundled inside archive; step-by-step for non-networked machines.
17. FR-M14-017: bundle assembly SLA: national bundle available within 4 h; constituency bundle within 30 min of tally finalisation.
18. FR-M14-018: download SLA: p95 latency <= 10 s (CDN-cached); 99.9% availability during petition window.
19. FR-M14-019: token inclusion proof SLA: p95 <= 500 ms.
20. FR-M14-020: bundle signature verifiable with standard OpenPGP and minisign tooling; no proprietary software required.
21. FR-M14-021: zero PII in bundle beyond what is statutorily published (station-level tallies, not voter identities).
22. FR-M14-022: multilingual web portal and CLI help text: English and Kiswahili.
23. FR-M14-023: WCAG 2.2 AA compliance for web portal; keyboard-navigable drag-and-drop zone; screen-reader labels on all status outputs.
24. FR-M14-024: full audit trail: every bundle generation event logged to transparency log (P05) and signed by HSM.
25. FR-M14-025: every bundle assembly run produces a deterministic build log; log hash included in manifest.
26. FR-M14-026: CDN cache invalidation on bundle publish; stale manifest requests receive 404 with redirect to current version.
27. FR-M14-027: rate limiting on bug-bounty endpoint: 10 submissions per IP per hour; CAPTCHA on web form.
28. FR-M14-028: bug-bounty findings that pass triage trigger `eel.transparency.bug_bounty_submitted.v1` event; downstream alerting.
29. FR-M14-029: bundle format documented in `docs/bundle-format-spec.md`; spec versioned alongside bundle semver.
30. FR-M14-030: CLI verifier exits with code 0 for PASS, 1 for verification failure, 2 for corrupt archive; machine-readable JSON output option (`--json`).
31. FR-M14-031: web portal shows per-component verification progress bar; completed components listed with hash and status.
32. FR-M14-032: REST API returns OpenAPI 3.1 schema; Swagger UI hosted at `/transparency/v1/docs`.
33. FR-M14-033: all REST endpoints return `X-Request-Id` and `X-Trace-Id` headers for correlation.
34. FR-M14-034: bundle download supports HTTP range requests (resume on interruption).
35. FR-M14-035: MinIO multipart upload for archives > 5 GB; streaming to CDN origin without full local copy.
36. FR-M14-036: bundle assembly idempotent: re-triggering for same election+version returns existing bundle_id without re-assembling.
37. FR-M14-037: correction bundle: new semver patch, delta manifest listing changed components, chain-of-custody record linking prior version.
38. FR-M14-038: constituency bundle references national Merkle root so verifier confirms it is a proper subset.
39. FR-M14-039: CLI verifier can verify constituency bundle standalone or against a downloaded national bundle.
40. FR-M14-040: batch pipeline emits `eel.transparency.bundle_published.v1` after CDN propagation confirmed; downstream systems key off this event.
41. FR-M14-041: KBC SFTP credential rotation: secrets fetched from OpenBao at deploy time; no hardcoded credentials.
42. FR-M14-042: Gazette API integration: POST notification with bundle URL and SHA-256; retry on 5xx; idempotent request key.
43. FR-M14-043: health endpoint (`GET /healthz`) reports: bundle assembly queue depth, last assembly timestamp, KBC mirror status, Gazette mirror status.
44. FR-M14-044: admin API (`POST /transparency/v1/bundles/assemble`) guarded by JWT with `bundle:assemble` SpiceDB permission; audit logged.
45. FR-M14-045: SBOM scanner in CI blocks on secrets, license violations (GPL-incompatible), and known CVEs above CVSS 9.
46. FR-M14-046: cross-arch reproducibility test runs in CI GitHub Actions matrix (x86_64 + aarch64 runners); build hashes compared and fail if divergent.
47. FR-M14-047: bundle verification in web portal works offline after first load (service worker caches WASM and static assets).
48. FR-M14-048: token proof endpoint returns proof in JSON-LD format compatible with W3C Verifiable Credentials for interoperability.
49. FR-M14-049: all inter-service calls from bundle-batch to RTS back-end (M10) authenticated with mTLS; certificate rotation handled by platform.
50. FR-M14-050: observability: bundle assembly duration, component count, archive size, CDN propagation time tracked in Grafana; Prometheus metrics exported.
51. FR-M14-051: SLO alerts: burn-rate alert if 99.9% budget consumed faster than 1 h; assembly > 4 h triggers PagerDuty P1.
52. FR-M14-052: runbook `docs/runbooks/transparency-svc-bundle.md` covers: assembly failure, HSM unavailability, KBC SFTP timeout, correction bundle procedure, CDN purge.

## Acceptance criteria

AC-M14-01: an independent third party on a clean machine with only the published bundle and open-source tooling can re-derive the final national tally and the result matches the published figures (reproducibility audit).
AC-M14-02: given a valid participation token SHA-256, the token proof endpoint returns a Merkle path that the CLI verifier confirms as valid inclusion in the published root.
AC-M14-03: statutory mirror: bundle appears on KBC SFTP and Gazette API within 1 h of `eel.transparency.bundle_published.v1` emission (integration test against doubles).
AC-M14-04: CLI verifier exits 0 on a known-good bundle and exits 1 on a bundle with a single tampered byte, logging the diverging component hash.
AC-M14-05: CI matrix build on x86_64 and aarch64 produces archives whose SHA-256 hashes are identical (reproducibility build test).
AC-M14-06: bug bounty submission via REST API returns HTTP 202; a subsequent GET with the anonymous token shows status `received`; IEBC triage interface lists the finding.
AC-M14-07: token proof endpoint sustains 99.9% availability across a 24 h load test simulating petition-window traffic (SLO measurement).
AC-M14-08: web portal verification completes successfully in-browser for a 500 MB bundle with zero network requests after initial page load (service worker test).
AC-M14-09: correction bundle is issued; prior version remains downloadable and its manifest unchanged; correction bundle manifest references prior version bundle_id.
AC-M14-10: constituency bundle verifies as a valid subset of the national bundle using CLI verifier (`--national-bundle` flag).
AC-M14-11: SBOM scanner blocks a CI build that introduces a dependency with a known CVE above CVSS 9 (regression gate test).
AC-M14-12: WCAG 2.2 AA audit (axe-core) on web portal reports zero critical or serious violations; keyboard-only navigation completes bundle verification end-to-end.
AC-M14-13: bundle assembly batch is idempotent: triggering assembly twice for the same election+version returns the same bundle_id and no duplicate components are stored.
AC-M14-14: bundle download endpoint supports HTTP range requests: a client that resumes mid-download receives a correct complete archive (integration test).

## Applicable edge cases

EC-DAT-01: duplicate `POST /transparency/v1/bundles/assemble` trigger; assembly is idempotent; second call returns existing bundle_id without re-run.
EC-DAT-02: Form 34A image arrives after bundle assembly starts; late-arriving component detected; assembly paused and restarted with complete set.
EC-DAT-03: component hash mismatch between RTS source and bundle manifest at inclusion time; assembly aborted; alert raised; no partial bundle published.
EC-DAT-04: bundle archive exceeds 50 GB; MinIO multipart upload used; streaming to CDN origin; progress tracked in metrics.
EC-DAT-05: constituency bundle references a Merkle root that has not yet been anchored; assembly deferred until anchoring event received.
EC-DAT-06: duplicate participation token SHA-256 submitted to proof endpoint; returns same proof path; no duplicate log entries.
EC-DAT-07: MinIO large object upload interrupted mid-stream; multipart resume on reconnect; no partial object visible to download endpoint.
EC-DAT-08: delta manifest for correction bundle lists a component that was not in the prior version; schema validation rejects it; operator alerted.
EC-DAT-09: bug bounty submission body exceeds 64 KB; request rejected with HTTP 413; no partial record written.
EC-DAT-10: bundle format version in manifest does not match current parser; CLI verifier reports version mismatch and exits 2.
EC-DAT-11: SBOM file missing from archive root; bundle assembly validation catches absence; assembly fails with structured error.
EC-DAT-12: zstd decompression produces output larger than declared size in header; extraction aborted; tampered archive reported.
EC-DAT-13: two assembly workers start simultaneously for the same election+version; database advisory lock ensures only one proceeds; the other returns existing bundle_id.
EC-DAT-14: CDN propagation confirmation times out after 30 min; `bundle_published` event withheld; operator alerted; manual confirm available via admin API.
EC-DAT-15: component file list from RTS returns empty for a constituency; assembly skips that constituency with a warning; national bundle proceeds.
EC-DAT-16: token proof requested for a token not in any Merkle tree; endpoint returns HTTP 404 with structured error; no 500 leak.
EC-SEC-01: bundle signature verification fails on download; CLI rejects with exit 1; web portal shows red banner; no tally displayed.
EC-SEC-03: HSM unavailable when bundle signing is attempted; assembly paused; PagerDuty alert; no bundle published without valid signature.
EC-SEC-05: JWT presented to admin assemble endpoint lacks `bundle:assemble` SpiceDB permission; HTTP 403 returned; attempt logged.
EC-SEC-09: CDN signed URL expires before download completes; client receives HTTP 403 on range resume; client refreshes URL via API and retries.
EC-SEC-12: CDN cache poisoning attempted on bundle manifest; cache key includes manifest signature version; poisoned response fails signature check.
EC-SEC-14: WASM verifier in web portal loaded from a tampered CDN; Subresource Integrity hash mismatch causes browser to block load; user sees error.
EC-SEC-16: secrets pattern detected in SBOM by CI scanner; build blocked; alert to security team; no bundle assembled from affected artifact.
EC-SEC-17: brute-force on bug-bounty anonymous token; rate limiter triggers at 20 requests per minute per IP; lockout with 429.
EC-SCL-01: 10 000 concurrent token proof requests during petition window; horizontal scaling via Kubernetes HPA; no request queued > 1 s.
EC-SCL-02: 1 000 simultaneous bundle downloads from CDN edge; origin not hit; CDN absorbs load; p95 latency <= 10 s.
EC-SCL-04: bundle assembly queue backs up with 50 constituency bundles simultaneously; worker pool processes in parallel up to concurrency limit; no data race.
EC-SCL-06: national bundle > 50 GB; MinIO multipart upload; streaming download via HTTP range; memory footprint per worker < 512 MB.
EC-SCL-08: bug bounty intake receives 500 submissions in 1 min (coordinated campaign); rate limiter and CAPTCHA throttle to sustainable rate; no DB overload.
EC-IDN-02: operator account used to trigger assembly is deactivated mid-run; in-flight assembly completes under service account; subsequent admin calls with same JWT rejected.
EC-IDN-10: citizen queries token proof using a token issued for a different election_id; endpoint returns 404; no cross-election data leak.
EC-LEG-08: correction bundle issued after statutory deadline; system records operator justification and timestamp; correction event logged to transparency log with reason field.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | bundle manifest builder, hash chain, token proof serialiser, bug-bounty FSM | nextest | M14-UNIT- |
| property | same inputs -> same hash; all components in manifest accounted for | proptest | M14-PROP- |
| contract | M10 form pull (consumer); M13 token proof (provider) | Pact | M14-PACT- |
| integration | M10 double; SoftHSM sign; MinIO; KBC SFTP double; Gazette double | nextest + testcontainers | M14-INT- |
| e2e | assemble bundle -> CLI verify -> public download -> bug bounty submission | cargo test + Playwright | M14-E2E- |
| build | cross-arch reproducibility: build on x86_64 and aarch64; compare hashes | CI (GitHub Actions matrix) | M14-BUILD- |
| load | national bundle assembly under load; download at CDN edge | k6 | M14-LOAD- |
| security | SBOM secret scan; signature bypass; OWASP ZAP; Semgrep | Semgrep + OWASP | M14-SEC- |

## Runbooks and dashboards

- docs/runbooks/transparency-svc-bundle.md (create): bundle assembly failure; HSM fail; KBC SFTP timeout; correction bundle procedure.
- platform/observability/transparency-svc/grafana-bundle.json: bundle assembly progress, token proof latency, download rate, bug bounty queue depth.
- platform/observability/transparency-svc/slo-alerts.yaml: 99.9% burn rate; assembly > 4 h; KBC mirror > 1 h.

## Open questions

- Cross-arch determinism: Rust reproducible builds are well-supported but the Python ICR model artifacts (M10) need verification that they produce identical byte output on x86_64 and aarch64.
- KBC SFTP access: KBC has historically required manual coordination for SFTP credential rotation; formal SLA needed before G4.
