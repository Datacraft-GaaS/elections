# portal-svc threat model

Module M11, WP-3.9. Last reviewed: 2026-08-21.

## Trust boundary

Edge (nginx + Varnish + Coraza WAF) sits in front of all three surfaces.
portal-api is inside the Cilium mesh; Elixir and Astro deploy images are edge-cached.

## Threats and mitigations

| Threat | Attack vector | Mitigation |
|---|---|---|
| Cache poisoning (EC-SEC-12) | Attacker replaces Merkle root in cached JSON | X-Eel-Merkle-Root and X-Eel-Snapshot-Id headers; cache key includes both; client WASM re-verifies |
| Stale snapshot injection (EC-TIM-05) | Serve outdated results without labelling | Stale flag in SnapshotSummary; UI shows last-updated timestamp; 60 s alert |
| DDoS on origin (FR-M11-021) | L3/L4 flood or L7 slowloris | 95% absorbed at edge (ADR-0010); static snapshots continue on origin loss |
| Proof endpoint abuse (EC-SEC-09) | High-rate scraping of /verify | Per-IP + per-/64 rate limit at Envoy Gateway and Coraza; portal-api returns 200 even under burst, never 5xx |
| PII leak (FR-M11-026) | Developer adds voter_id or name to response struct | All response types annotated pii: false in data_dictionary.yaml; CI Semgrep Tier A/B log ban |
| SSRF via civic CMS (EC-SEC-06) | Attacker sets URL field to internal IP | Allowlist in CMS (WP-5.7); not in scope for portal-api public endpoints |
| Injection (EC-SEC-05) | SQL injection via snapshot_id query param | UUID parse before DB access; parameterised queries; no raw string interpolation |
| Snapshot with mismatched cosign sig | Attacker pushes tarball with altered Merkle root | eel-web-astro loader cosign-verifies before render; portal-api reject on sig mismatch; test in integration_test.rs |

## Out of scope for WP-3.9

- crates/gazette, crates/inspection, astro/explainer, py/tts (WP-5.7).
- CMS admin path (Django, separate threat model).
- Kit attestation (WP-3.8).
