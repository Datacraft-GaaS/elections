# 8. Definition of Done per work package

1. Unit coverage thresholds met per language; lcov uploaded.
2. Property tests for every parser, FSM, validator, codec in scope; IDs in `TRACEABILITY.yaml`.
3. Every applicable §6 row mapped to ≥1 passing tagged test.
4. Pact green with all consumers/providers; `can-i-deploy` pass.
5. Events registered in Karapace; compat pass; `docs/EVENTS.md` changelog; idempotency + signature-verify consumer tests.
6. OpenAPI published; `openapi-diff` clean; generated clients compile + smoke in 4 languages.
7. Idempotency proven on every write endpoint (auto-generated EC-DAT-01/02).
8. Degradation modes implemented, flag-switchable, tested; degradation runbook.
9. Resilience layer used everywhere; Semgrep no-bypass rule passes.
10. Zero critical/high SAST/SCA/container/DAST; ASVS checklist signed.
11. STRIDE threat model with mitigations mapped to tests.
12. `data_dictionary.yaml` complete; PII wrappers; log-scanner passes.
13. Fuzz targets for owned boundaries; ≥1 h CI fuzz clean; corpus committed.
14. Formal models updated where tally/declaration/referendum/freeze/Merkle touched.
15. Load test at module SLA in local profile; pre-prod when available; ≤10% p99 regression.
16. Owned chaos scenarios added and passed in staging.
17. i18n externalised; 11 locales + pseudo pass; no hardcoded strings.
18. axe zero violations; manual screen-reader script signed; AAA for M68.
19. Grafana dashboards + PrometheusRule SLO burn alerts shipped.
20. Mutation threshold met.
21. Reproducible build bit-identical on 3 targets; SBOM signed ≤1 h.
22. Mobile/kit: attestation, RASP, secure storage, version floor, N-1→N migration tests.
23. Runbooks: deploy, rollback (≤15 min proven), DR, on-call updated.
24. Docs: README, API, data-flow diagram, EVENTS.md, CHANGELOG.
25. Completeness scan (§9.1) zero hits in WP paths.

