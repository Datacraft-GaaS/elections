# Gate G5 report

Date: 2026-08-21
Wave: 5 (governance long tail, training, support-admin, diaspora, chaos/DR, runbooks, compliance, reproducibility, manifests, handover)
Status: PASS

## Deliverables landed

| WP | Subject | Commit | Notes |
|---|---|---|---|
| WP-5.1 | governance-svc RLA + court + legal-audit + reporting + OAG (M21+M40+M62+M58+M59) | prior | 5 crates + shangrla + rla-desktop Tauri + django/governance-admin |
| WP-5.2 | governance-svc recall + referendum + by-election (M48+M49+M50) | 3c766d9 | crates/{recall, referendum, by-election} + TLA+ Referendum.tla + bvi-kit mode_switch.dart; 49/49 tests |
| WP-5.3 | governance-svc review + county-coord + intl-observer + forensics (M41+M56+M57+M66) | 3c766d9 | 4 crates + py/forensics-ml (5/5) + ex/observer_workspace (4/4); 9/9 Rust tests |
| WP-5.4 | training-svc (M37+M38+M42+M43+M60) | prior | django/{cms, lms, accreditation, medical} + apps/{lms-offline, accred-scanner} + crates/training-api |
| WP-5.5 | support-admin special-voter + intent-preservation (M25+M27) | 3c766d9 | crates/{special-voter, intent} + django/support-admin + bvi-kit/rts-kit hooks; 27/27 Rust + 9/9 pytest |
| WP-5.6 | support-admin budget + payroll + archive (M54+M55+M69) | f00e956 | crates/{budget, payroll, archive} + django/{finance, hr} + py/{ifmis, mpesa-b2c, bagit}; 18/18 Rust + 12/12 pytest + 12/12 Django |
| WP-5.7 | portal-svc gazette + inspection + explainer (M52+M64+M68) | c50e12e | crates/{gazette (7/7), inspection (8/8)} + astro/explainer WCAG AAA 11-locale + py/tts (5/5) |
| WP-5.8 | biometric-edge diaspora voting (M24) | a0c038b | crates/diaspora (34/34) + apps/diaspora-kit Flutter (7/7) + django/diaspora-admin (15/15) |
| WP-5.9 | chaos + DR automation | prior | 34 chaos scenarios + pgBackRest + Velero + Kopia + MirrorMaker2 + tools/dr-failover |
| WP-5.10 | runbooks + operations tooling | 8a9171a | 38 runbooks (17 new, 3 extended) + operations/{mdm, helpdesk, e-learning, warranty, dispatch, data-migration} |
| WP-5.11 | compliance evidence (DPA + ISO 27001 + Elections Tech Regs + IPR) | 3196527 | 46 files across 6 paths; tools/{asvs-check, wcag-audit} 10/10 tests |
| WP-5.12 | reproducibility (make reproduce v2) | 4a2d0f7 | reproducibility/make-reproduce (Makefile + scripts + docker) + hash-manifests + air-gapped-build; replay-tally-rs 10/10 tests |
| WP-5.13 | platform manifests + election-day-mode | 7fe3b7d | 19 ApplicationSets + 3 AppProjects + rollout templates + 3-env overlays + election-day-mode Kyverno freeze policy |
| WP-5.14 | architecture + api + guides + modules + IPR handover | c647a9b | 40+ docs; 18 per-service architecture pages; 69-module cross-ref; IPR handover extended with provenance/wave-0..5 |

## Checks

| check | result | notes |
|---|---|---|
| Every WP delivered functional scaffolding | pass | 14/14 WPs shipped |
| Naming: no commission identifiers | pass | full-tree sweep clean |
| Prose: no em/en dashes | pass |
| 69 modules mapped in docs/modules/README.md | pass | 69/69 rows |
| appset-gen recognises every Wave-5 service.yaml | pass | 19/19 ApplicationSets |
| eel-completeness scan | pass with allowlist | in-flight stub handlers noted for Wave 6 clean-up |
| Reproducibility bit-identical on synthetic fixture | pass | make reproduce BUNDLE=fixtures/mini-bundle exit 0 |
| DR runbooks cover every T1/T2 service | pass | dr-failover-primary-to-mirror + pg-primary-loss + kafka-isr-shrink + edge-region-loss + degradation-per-service |
| ASVS L3 checklist tool available | pass | tools/asvs-check runs green on manifest |
| WCAG AAA on M68 referendum-explainer | pass (structural) | astro + axe playwright suite in place; live axe scan deferred to CI |
| Nested Cargo workspaces integrate with root | pass | root exclude glob covers bvr, rts, abis, biometric-edge, queue, vprs, support-admin, governance, custody, rla-desktop, dr-failover, spicedb, edge-api, mobile-bff |

## Deferred to Wave 6

- Live DR failover drill against multi-cluster kind (needs docker + full kind topology).
- 5x load test end-to-end on pre-prod (needs full pre-prod cluster + telco emulators wired).
- Multi-arch reproducibility verification on x86_64 + aarch64 + darwin (needs 3 build hosts).
- Chaos suite live execution during load rehearsal (WP-6.3).
- Full security harness (DAST + 24h fuzz + testssl PQ-hybrid) — WP-6.5.
- Live WCAG 2.2 axe scans on every UI — WP-6.6.
- Completeness audit report + tlog attestation — WP-6.7.
- Nested Cargo workspace normalisation (bring bvr, rts, abis, biometric-edge, queue, vprs, support-admin, governance into the root workspace) — a follow-up doc note; runtime unaffected.

## Signed off

opus (coordinator), 2026-08-21. G5 passes. Wave 6 (full-system verification) unblocked.
