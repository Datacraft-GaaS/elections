# 9. Completeness audit procedure (WP-6.7, also run as blocking CI from Wave 0)

| Step | Mechanism | Fail condition |
|---|---|---|
| 9.1 Stub scan | `tools/completeness/scan` (tree-sitter + regex): `todo!()`, `unimplemented!()`, TODO/FIXME/XXX, `NotImplementedError`, `pass`/`...`-only bodies, `UnimplementedError`, `@tag :skip`, `#[ignore]`, `pytest.mark.skip`, `it.skip`/`xit`/`test.skip`, 501 handlers, OpenAPI paths without bound handler, event schemas without producer+consumer tests, default-off flags without enabling test | any hit outside allowlist (only ClickHouse deferral, with ADR ref + expiry) |
| 9.2 Surface inventory | per Appendix-A surfaces column, assert deployable artefact exists (chart, APK/IPA, Tauri bundle, Astro build, USSD menu, CLI) | missing artefact |
| 9.3 Traceability matrix | union of module×feature (FR-IDs from `sw/modules`), ACCEPTANCE-CRITERIA criteria, SECURITY SR-* controls, §6 EC rows → test IDs from tags → JUnit results | any row with zero mapped or any failing |
| 9.4 Contract closure | Pact `can-i-deploy` all pairs at tag; every Karapace schema has ≥1 producer + ≥1 consumer test | unverified pair |
| 9.5 Endpoint closure | every OpenAPI path+method: contract test, idempotency test (writes), DAST pass, 4-language client smoke | uncovered path |
| 9.6 Runbook closure | every service has deploy/rollback/DR/degradation runbooks + chaos scenario executed ≤30 days | missing/stale |
| 9.7 SLA closure | simulator report measures and meets every Appendix-A number | unmeasured/failed |
| 9.8 Legal closure | rules-engine table maps every statute citation (§§6A, 14A, 30, 39(1C), 44, 44A, 53-55, 69, 72, 78, 80, 83-84, Art. 87/104, §75(1)) to a test | unmapped citation |
| 9.9 Report | `docs/audit/completeness-report.md` + signed JSON attestation to tlog meta-events | build red on any failure |


## 10. Verification of this plan's execution (end-to-end)

1. After docs land: `ls docs/plans` shows 12 files + `modules/` with 69 pages + `TRACEABILITY.yaml`; every module slug in `modules/` matches `docs/modules/README.md`.
2. After each wave: `docs/audit/gate-G<n>.md` exists with CI links; `just ci` green on `main`; `tools/completeness/scan` zero hits in that wave's paths.
3. After Wave 3: `just up && just rehearse --scale 1/100` runs a kit→RTS→declaration→portal flow and prints inclusion proofs that `verifier-rs` validates.
4. After Wave 6: completeness report shows 69/69 modules, 118/118 EC rows, all SLAs, all statutes mapped and green; attestation visible via `GET /api/v1/log/inclusion/<leaf>`.

