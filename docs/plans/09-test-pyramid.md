# 7. Test pyramid and harnesses

| Layer | Rust | Python | Elixir | Flutter | TS | Gate |
|---|---|---|---|---|---|---|
| Unit | cargo-nextest | pytest | ExUnit | flutter_test | vitest | ≥85% line / 75% branch; election-critical 90/80 |
| Property | proptest + kani | hypothesis | StreamData | custom gens | fast-check | every parser, FSM, validator, codec, HLC, Merkle, idempotency |
| Contract | Pact | pact-python | pact_elixir | pact_dart | pact-js | every ADR-0020 service pair; Karapace compat; openapi-diff; buf breaking |
| Integration | testcontainers-rs | testcontainers-python | Divo | integration_test vs compose | none | PG17+Patroni, Kafka+Debezium+Karapace, Valkey, MinIO, OpenSearch, Keycloak, SpiceDB, OpenBao, SPIRE, SoftHSMv2, toxiproxy, all integration doubles |
| E2E | kit-simulator | none | Playwright | patrol | Playwright, tauri-driver | 1..65k virtual kits incl. legacy shims |
| Fuzz | cargo-fuzz | atheris | none |, | none | envelope + RFC 8785; Ed25519/ML-DSA verify; Merkle proof decode; CBOR/JSON/zstd; MRZ; SMS/USSD codec; QR badge/seal; shapefile/GeoJSON; PDF/image (ICR); PKCS#11 responses; attestation tokens; SAML/OIDC; problem+json/ETag; Idempotency-Key; HLC string; legacy envelope; timestamp parser; archive extractor |
| Formal | TLA+/Apalache: tally 34A→B→C + exactly-once + amendments; referendum threshold; run-off/tie; register-freeze race. kani/creusot: Merkle append/inclusion/consistency, HLC merge, monotonic counter | | | | | nightly CI; independent review |
| Load | k6 + Gatling; Rust election-day simulator | locust (ML) | | | | local 1/100 per WP; pre-prod 1x and 5x; regression gate |
| Chaos | Chaos Mesh list in 5.9 | | | | | weekly staging; monthly pre-prod; T-7 full |
| Security | cargo-audit/deny, Semgrep (the Commission rules), Trivy, Grype, cosign verify, ZAP, Nuclei, testssl.sh, ASVS L3/L2 | Bandit, pip-audit | Sobelow, mix_audit | MobSF | npm audit | zero critical/high; ASVS signed per WP |
| Accessibility | none |, | axe via Playwright | semantics tests | axe | AA all; AAA M68; manual SR scripts |
| i18n | none |, | gettext snapshot | ARB snapshot | i18next snapshot | pseudo-locale 150% + 11 locales; zero missing keys |
| Reproducibility | Nix-pinned, `--locked`, SOURCE_DATE_EPOCH, diffoscope | hashes | mix.lock | pinned | frozen | bit-identical on 3 targets; M14 cross-arch |
| Mutation | cargo-mutants ≥70% (≥80% critical) | mutmut ≥70% | muzak ≥60% | none | Stryker ≥60% | weekly; no regression |

