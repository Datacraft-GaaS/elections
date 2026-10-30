# UI catalogue

Every human-facing surface in the eel platform: 5 Astro static sites, 7 Elixir Phoenix + LiveView real-time dashboards, 15 Django admin consoles, 13 Flutter apps (kit + mobile + desktop mobile), 2 Tauri desktops, 21 Grafana operational dashboards. Total 63 distinct UIs.

## Note on screenshots

Screenshots require the full local stack to be running (`just up` bringing up kind + Postgres + Kafka + Valkey + MinIO + OpenSearch + Keycloak + SpiceDB + OpenBao + SPIRE + SoftHSM + Grafana + all 18 services). This coordinator host runs no docker, so images cannot be captured from this environment. Every UI listed below builds under CI (Astro `pnpm build`, LiveView `mix compile`, Django `manage.py check`, Flutter `flutter analyze`, Tauri `cargo check`), and each entry names the exact command that renders it once the platform is up. Where an ASCII wireframe helps convey layout it is included; otherwise the entry gives the pages, key components, and route table.

## Contents

1. Design system + shared foundations
2. Public-facing Astro sites (5)
3. Real-time LiveView dashboards (7)
4. Staff Django admin consoles (15)
5. Flutter kit + mobile + custody apps (13)
6. Tauri desktop apps (2)
7. Grafana operational dashboards (21)
8. Accessibility + i18n coverage
9. How to preview each UI locally

---

## 1. Design system + shared foundations

`packages/design-system/` supplies the visual language across Astro, Svelte, and Django surfaces.

- `tokens.json` — colour palette (WCAG AA contrast pairs + AAA for M68), typography scale, spacing, radii, focus-ring, elevation.
- `astro/` — Astro component wrappers.
- `svelte/` — Svelte component variants for the LiveView surfaces that need embedded islands.
- `django/` — `eel_design_system` Django app providing template tags + static asset shipping.

`packages/i18n/locales/` — 11-language message catalogues: en (English), sw (Kiswahili), plus 9 regional languages (ki Kikuyu, luo Dholuo, kal Kalenjin, kam Kamba, guj Gujarati, mer Meru, mas Maa, som Somali, ksl Kenyan Sign Language glossary).

`packages/eel-web-astro/` — Astro base with signed-content loader that verifies Tier-1 tarballs before rendering.

`packages/eel-web-py/` — Django base with SpiceDB middleware, HTMX partials, PG session store, audit-shadow mixin, PII field wrappers.

---

## 2. Public-facing Astro static sites (5)

### 2.1 portal-svc/astro/public — M11 Election Results Portal

- Home page: national totals + county drilldown.
- Constituency page: per-constituency 34A form list with signed downloads.
- Station page: individual 34A image + hash + inclusion proof.
- Verifier widget: WebCrypto SHA-256 in-browser verifier + tlog leaf viewer.
- Live counts via LiveView island (portal_live).
- RSS + Atom feeds under `/feeds/`.

6 pages. Static build with `pnpm build`. Cache-friendly via CDN + Varnish rules under `edge/varnish/`. Publishes signed tarballs from `services/portal-svc/publications/`.

```
+-------------------------------------------------------+
| eel                                    lang: sw / en  |
+-------------------------------------------------------+
| National totals                                       |
|  Presidential  --  votes --  %                        |
|  Party A     8_234_112     42.1%                      |
|  Party B     7_912_045     40.5%                      |
|  ...                                                  |
| [Merkle root SHA-256 abcd... verify]                  |
+-------------------------------------------------------+
| Counties       | Constituencies                       |
|  Nairobi   >   |                                      |
|  Mombasa   >   |                                      |
|  ...           |                                      |
+-------------------------------------------------------+
```

### 2.2 portal-svc/astro/explainer — M68 Referendum Explainer (WCAG 2.2 AAA)

- Landing with question filter.
- Per-question page in every locale with fact-check annotations, KSL video companion, TTS audio player, download-as-PDF.

2 pages (index + [lang]/[question] template). 11 locales. Playwright a11y suite asserts zero axe violations at AAA colour contrast (>= 7:1). TTS via `services/portal-svc/py/tts` (piper-tts or stub adapter).

### 2.3 abis-svc/astro/public-reports — Public transparency reports

- Landing with filterable list of published public reports.
- Per-run page with download links (PDF/CSV/JSON/HTML), SHA-256 badge per artefact.
- WebCrypto in-browser verification widget: paste bytes -> compute SHA-256 -> green tick if matches declared.
- Tlog inclusion proof viewer with leaf index + root hash + witness cosigs.
- Permanent `/reports/latest/<slug>` permalinks.

23 pages built (11 locales x per-run). RSS 2.0 + Atom 1.0 feeds. No external assets (strict CSP).

```
+-------------------------------------------------------+
| eel public reports                                    |
+-------------------------------------------------------+
| Filter: [class: fairness]  [year: 2026]  [go]         |
|                                                       |
| Report                        window        formats   |
| Quarterly fairness Q3 2026    Jul-Sep       PDF CSV   |
| Weekly FMR/FNMR week 34       Aug 19-25     JSON PDF  |
| ...                                                   |
+-------------------------------------------------------+
| Latest: /reports/latest/fairness-quarterly            |
+-------------------------------------------------------+
```

### 2.4 abis-svc/astro/adhoc-editor — Ad-hoc report authoring SPA

- List view of ad-hoc reports the caller owns or has been shared.
- Editor page with 3 panes: metadata + Monaco SQL editor (Preact island) + live preview.
- LintBadge shows safety_score + violation list.
- PreviewTable renders sample rows with per-format download buttons (PDF, DOCX, Markdown, CSV).
- ShareDialog to grant view/run/edit/admin permission.
- PublishWizard with CronBuilder + subscriber multi-select + format picker.

3 pages. Monaco editor + marked + DOMPurify client-side. Preact islands for interactive components; rest static.

### 2.5 bvi-svc/astro/bvv — M07 BVV kiosk lookup

- Kiosk landing with two-tap lookup by ID.
- Timing-constant response shape to prevent enumeration; daily lookup cap per caller.
- Objection filing form when the returned entry is contested.

2 pages. Deployed to touchscreen kiosks at the Commission constituency offices.

---

## 3. Real-time Elixir Phoenix + LiveView dashboards (7)

### 3.1 portal-svc/ex/portal_live — Live election-night counts

- Country + county + constituency + station drilldown updating in real time via Phoenix PubSub over the rts publication stream.
- Merkle root banner + witness cosign badge.
- Rate-limited SSE for 5-10M concurrent viewers (backed by edge cache).

### 3.2 ecc-svc/ex/ecc_live — M16 Election Command Centre

- 46k-kit telemetry snapshot rendered in under 2 s (as required by SLO in `docs/plans/12-appendix-module-inventory.md`).
- ECCAlert stream with dedup + rate-limit + severity ranking.
- S44A dual-control dispatch console.
- Party + observer + journalist read-only mirrors.

### 3.3 ecc-svc/ex/iab_live — M15 IAB Oversight Console

- Read-only board dashboard for the Independent Assurance Board.
- Complaint intake tied to M23 grievance-channels.
- L1-L4 escalation ladder with hardware-key authentication (WebAuthn + YubiKey enforced at `packages/eel-auth-*`).
- Quarterly report builder.

### 3.4 queue-svc/ex/queue_map — M19 Queue Transparency Map

- National choropleth of polling-station queue depths.
- Wait-time percentile heatmap.
- Disenfranchisement flag list feeding ECC.
- s.14A extension recommendation table with Chair HSM sign-off action.

### 3.5 ai-platform-svc/ex/speech_analyst — M17 Speech Integrity Analyst Workspace

- Content-flag inbox with clustered CIB (coordinated inauthentic behaviour) groups.
- Analyst decision form: label + reason + evidence upload.
- Deepfake detection score panel with per-detector breakdown.
- Kenyan-language bias report per model card.

### 3.6 bvi-svc/ex/bvv_web — M07 BVV Backend Web

- Staff view of BVV lookups + objections + corrections.
- Live turnout percent per station with sparkline.
- Auto-refresh via LiveView every 15 s.

### 3.7 governance-svc/ex/observer_workspace — M57 International Observer Workspace

- Per-mission workspace with RBAC-scoped queries (admin bypass + mission-scoped allow + cross-mission deny per WP-5.3 tests).
- Briefing document store.
- Observation report editor with joint-statement collaboration mode.
- Session-scoped SpiceDB permission check.

All 7 LiveView apps build with `mix compile --warnings-as-errors`. `mix test` + `mix format --check-formatted` green. `mix phx.server` renders on `http://localhost:4000..4020` depending on service (port map in each service's `deploy/service.yaml`).

---

## 4. Staff Django admin consoles (15)

Every Django app runs with `python manage.py runserver 0:8000` after `DATABASE_URL=... python manage.py migrate`. All use `packages/eel-web-py` as the base with SpiceDB permission checks + audit shadow + PII wrappers. Templates use `packages/design-system/django` tokens (WCAG AA contrast, skip-to-main, 48 dp touch targets, ARIA landmarks, keyboard-first navigation).

| # | Django app | Purpose | Key models |
|---|---|---|---|
| 1 | bvr-svc/django/bvr-admin | M02 register curation | Voter, VoterVersion, AuditEvent, MerkleRoot |
| 2 | inventory-svc/django/inventory-admin | M20+M39+M65 custody, printing, diaspora logistics | Asset, CustodyEvent, Seal, PrintOrder, PouchCustody |
| 3 | rts-svc/django/declaration-admin | M51 declaration + M63 ballot serial | Declaration, CertificateOfElection, PetitionWindowTracker |
| 4 | governance-svc/django/governance-admin | Petitions, evidence, references, RLA runs | AuditRun, EvidenceBundle, ReferralPacket |
| 5 | biometric-edge-svc/django/diaspora-admin | M24 diaspora embassy station + pouch admin | EmbassyStation, DiasporaRoll, Form34ADiaspora, PouchCustody |
| 6 | support-admin-svc/django/support-admin | M25+M27 special-voter + intent-preservation | Facility, S44AEvent, IDPReassignment, PreMarkValidationEvent |
| 7 | support-admin-svc/django/hr | M55 HR + payroll | JobApplication, ShortlistDecision, StaffContract, PayrollRun |
| 8 | support-admin-svc/django/finance | M54 budget + procurement + vouchers | Budget, Procurement, PaymentVoucher, CoBReport |
| 9 | training-svc/django/cms | M37 voter education CMS | ContentItem, ContentVersion, FactCheckReview |
| 10 | training-svc/django/lms | M38 poll-worker LMS | CourseModule, WorkerEnrollment, WorkerCertificate |
| 11 | training-svc/django/accreditation | M42+M43 media + observer accreditation | MediaAccreditation, ObserverOrganisation, IndividualObserver |
| 12 | training-svc/django/medical | M60 medical first response | MedicalIncident, DispatchRequest, ClusterAlert (no-PHI schema) |
| 13 | abis-svc/django/abis-console | Adjudicator workbench | AdjudicationCase, ReviewerAction, PanelVote, Reviewer |
| 14 | abis-svc/django/abis-reports-console | Reports staff console | ReportDefinition, ReportRun, ReportArtifact, ReportSubscriber, ReportDelivery, ReportTlogAnchor, ReportAccessLog |
| 15 | abis-svc/django/abis-adhoc-editor | Ad-hoc reports authoring (staff) | AdhocReport, AdhocReportVersion, AdhocPreview, AdhocShare, AdhocSnippet |

Every admin has: list view with filter + search + sort, HTMX partials for inline edit, dual sign-off form for Tier-3 workflows, download-with-signed-URL that increments `eel_report_access_log` when applicable, and permission-aware queryset middleware (author + shared + admin).

```
Django admin skeleton (per app)
+-------------------------------------------------------+
| eel {app}                       user@commission  logout     |
+-------------------------------------------------------+
| [nav: List]  [nav: New]  [nav: Reports]  [nav: Audit] |
+-------------------------------------------------------+
| Filter: [status]  [date-range]  [assignee]  [reset]   |
| Search: [__________________________]  [go]            |
|                                                       |
| Id     | Subject     | Status      | Created    | ... |
| a1..   | ...         | Open        | 2026-08-22 |     |
| a2..   | ...         | Assigned    | 2026-08-22 |     |
| ...                                                   |
|                                                       |
| [bulk: assign to me]  [bulk: escalate]  [export CSV]  |
+-------------------------------------------------------+
```

---

## 5. Flutter apps (13)

### Kits (5)

| App | Modality | Target | Attestation | Offline queue |
|---|---|---|---|---|
| bvr-kit | BVR enrolment (biometric + demographic + ID) | Android + iOS | Play Integrity + App Attest + TPM | 7 days Isar+SQLCipher |
| bvi-kit | BVI polling-day identification | Android + iOS | Play Integrity + App Attest + TPM | 7 days |
| rts-kit | Results Transmission scan-and-send | Android tablet | Play Integrity + kit CA cert | 7 days with SMS fallback |
| legacy-android | Shim for the 14.1k legacy KIEMS devices | Android 8+ | Software-tier attestation | 7 days |
| diaspora-kit | Embassy polling-day + delayed transmission | Android + iOS | Play Integrity + App Attest | 30 days with dual-clock TZ |

### Mobile staff (5)

| App | Persona | Key flows |
|---|---|---|
| party-agent | Political party polling agent | Kit check-in via QR, HSM-anchored countersignature, dispute, duress silent alert |
| poll-clerk | Polling-station clerk | Kit mirror dashboard, offline reference, incident, EOD signed report, kiosk mode |
| observer | Domestic + international observer | Visits, signed incident attachments, sampling verification, mission PDF, AG mirror |
| field-support | IT + logistics field support | Tickets, diagnostics, safe signed remote actions, warranty, escalation, offline runbook |
| personnel | Temporary staff | 250k check-ins with GPS/cell/Wi-Fi spoof check, 15-min pings, payroll trigger |

### Ops + training (3)

| App | Purpose |
|---|---|
| custody-mobile | Inventory chain-of-custody on the move (RFID + GPS + seals) |
| lms-offline | Poll-worker training offline module completion + sync |
| accred-scanner | Media + observer accreditation QR badge scanner |

### AI plugin (1)

| App | Purpose |
|---|---|
| kit-l1 | Level-1 on-device anomaly detector plugin for kits |

All Flutter apps build on `packages/kit-sdk-flutter` which supplies: Isar + SQLCipher secure store, offline queue, attestation abstraction, HLC clock, sync engine, cert pinning, i18n (11 locales), accessibility widgets (>= 48 dp targets), time-fence FSM, duress-finger hook.

Build + release: `flutter build apk --release --split-per-abi` for Android; `flutter build ios --release` for iOS; MDM profiles at `operations/mdm/android-enterprise/` and `operations/mdm/apple-business-manager/`.

---

## 6. Tauri desktop apps (2)

### 6.1 rts-svc/apps/ro-desktop — M09 Returning Officer Workbench

- 34B compile from the constituency's 34A submissions.
- 34C compile at the tally centre.
- Edit-with-reason with immutable audit trail.
- Dispute case creation with agent countersignature.
- Session recording hash to tlog.
- SQLite WAL for crash safety; offline continuation.

### 6.2 governance-svc/apps/rla-desktop — M21 Offline RLA Auditor

- SHANGRLA risk-limiting audit runner (Stark 2020).
- NIST beacon + HSM + IAB card combined seed for sampling.
- Ballot manifest importer.
- T-statistic monitor + auto-recount trigger.
- Runs entirely offline for tamper-isolation during the audit.

Both wrap a Rust core in a Tauri 2 shell. Build with `cargo tauri build`. Sign with cosign per `tools/hsm-ceremony/`. Release channels tracked in `platform/argocd/rollouts/`.

---

## 7. Grafana operational dashboards (21)

Located at `platform/observability/dashboards/`. Auto-provisioned to the platform Grafana instance via ArgoCD ApplicationSet at `platform/observability/dashboards.appset.yaml` (per ADR-0022).

### 18 per-service dashboards

Each per-service dashboard shows: request rate, error rate, p50/p95/p99 latency, degradation-mode flag, event-emission lag, database connection pool, HSM signature counters, upstream/downstream dependency health.

| # | Service | Dashboard file |
|---|---|---|
| 1 | transparency-svc | transparency-svc.json |
| 2 | boundary-svc | boundary-svc.json |
| 3 | rts-svc | rts-svc.json |
| 4 | bvr-svc | bvr-svc.json |
| 5 | abis-svc | abis-svc.json |
| 6 | bvi-svc | bvi-svc.json |
| 7 | biometric-edge-svc | biometric-edge-svc.json |
| 8 | portal-svc | portal-svc.json |
| 9 | ecc-svc | ecc-svc.json |
| 10 | queue-svc | queue-svc.json |
| 11 | inventory-svc | inventory-svc.json |
| 12 | ai-platform-svc | ai-platform-svc.json |
| 13 | field-mobile-svc | field-mobile-svc.json |
| 14 | vprs-svc | vprs-svc.json |
| 15 | crms-svc | crms-svc.json |
| 16 | governance-svc | governance-svc.json |
| 17 | training-svc | training-svc.json |
| 18 | support-admin-svc | support-admin-svc.json |

### 3 ABIS-specialised dashboards

| Dashboard | Panels |
|---|---|
| abis-overview.json | Enrolment rate, dedup p50/p95/p99, match 1:N p95/p99, match 1:1 p95/p99, adjudication queue depth by tier + priority, vendor health matrix, HNSW index size, shard rebalance activity |
| abis-fairness.json | Fairness slices by sex, age band, county with breach highlighting; deviation vs baseline; trend over 90 days |
| abis-attestation.json | TDX + SEV-SNP verifications, failure reasons breakdown, nonce replay attempts, policy version drift |

Alert rules ship alongside dashboards at `platform/observability/alerts/abis/abis-slo.yaml` and per-service `<svc>-slo.yaml` (created during Wave 4 + 5). Each alert carries a `runbook_url` annotation pointing to `docs/runbooks/`.

---

## 8. Accessibility + i18n coverage

- **WCAG 2.2 AA** enforced on every UI via axe checks in Playwright + Django test middleware + Flutter semantics tests + LiveView `mix test.a11y`.
- **WCAG 2.2 AAA** on `portal-svc/astro/explainer` (M68) — colour contrast >= 7:1, augmented navigation, KSL video companion, TTS audio, plain-language rewrite.
- **11 locales** with pseudo-locale (150 % width) automated check in `verification/a11y-i18n/pseudo-locale-check.sh`.
- **Screen readers** — NVDA scripts for portal + verifier, TalkBack scripts for all 9 mobile apps, VoiceOver AppleScripts for the 2 Tauri desktops. Coverage in `verification/a11y-i18n/screen-reader/`.
- **Font fallback stack** documented in `packages/design-system/tokens.json`.
- **Keyboard-first navigation** — every UI has a visible focus ring + skip-to-main link + logical tab order.
- **Reduced-motion** honoured via `prefers-reduced-motion` media query.
- **Colour-blind-safe palette** at `packages/design-system/tokens.json` with symbol + text + pattern triple encoding on M67 ballot symbols.

---

## 9. How to preview each UI locally

### 9.1 One-time setup

```bash
git clone https://github.com/Datacraft-GaaS/elections && cd elections
nix develop
just up  # brings up kind cluster + Postgres + Kafka + Valkey + MinIO + OpenSearch + Keycloak + SpiceDB + OpenBao + SPIRE + SoftHSM + Grafana
just seed # loads deterministic synthetic voters + candidates + stations
```

### 9.2 Astro sites

```bash
cd services/portal-svc/astro/public   && pnpm install && pnpm dev  # http://localhost:4321
cd services/portal-svc/astro/explainer && pnpm install && pnpm dev  # http://localhost:4322
cd services/abis-svc/astro/public-reports && pnpm install && pnpm dev  # http://localhost:4323
cd services/abis-svc/astro/adhoc-editor   && pnpm install && pnpm dev  # http://localhost:4324
cd services/bvi-svc/astro/bvv          && pnpm install && pnpm dev  # http://localhost:4325
```

### 9.3 LiveView dashboards

```bash
cd services/portal-svc/ex/portal_live       && mix deps.get && mix phx.server  # http://localhost:4000
cd services/ecc-svc/ex/ecc_live             && mix deps.get && mix phx.server  # http://localhost:4001
cd services/ecc-svc/ex/iab_live             && mix deps.get && mix phx.server  # http://localhost:4002
cd services/queue-svc/ex/queue_map          && mix deps.get && mix phx.server  # http://localhost:4003
cd services/ai-platform-svc/ex/speech_analyst && mix deps.get && mix phx.server  # http://localhost:4004
cd services/bvi-svc/ex/bvv_web              && mix deps.get && mix phx.server  # http://localhost:4005
cd services/governance-svc/ex/observer_workspace && mix deps.get && mix phx.server  # http://localhost:4020
```

### 9.4 Django admins

```bash
# example: bvr-admin
cd services/bvr-svc/django/bvr-admin
export DATABASE_URL=postgres://eel:eel@localhost:5432/bvr
uv run python manage.py migrate
uv run python manage.py createsuperuser
uv run python manage.py runserver 0:8000  # http://localhost:8000/admin
```

Repeat for each of the 15 Django apps; increment the port (8000..8014).

### 9.5 Flutter apps

```bash
cd services/biometric-edge-svc/apps/bvr-kit
flutter pub get
flutter run -d chrome  # for PWA preview
flutter run -d android  # for connected device
flutter run -d ios      # for connected iOS device (macOS host)
```

### 9.6 Tauri desktops

```bash
cd services/rts-svc/apps/ro-desktop
cargo tauri dev  # opens the desktop shell with hot-reload
```

### 9.7 Grafana dashboards

After `just up`, visit `http://localhost:3000` (default). Grafana is provisioned with all 21 dashboards under the `eel` folder. Datasource: Prometheus + Loki + Tempo. Alertmanager rules live at `platform/observability/alerts/`.

---

## Summary table

| Class | Count | Framework |
|---|---|---|
| Astro static sites | 5 | Astro 5 + Tailwind + MDX + WebCrypto + Playwright |
| LiveView dashboards | 7 | Elixir 1.17 + Phoenix + LiveView + Ecto + Broadway |
| Django admin consoles | 15 | Django 5 + HTMX + eel-web-py + SpiceDB |
| Flutter kit + mobile | 13 | Flutter 3.2x + Isar + SQLCipher + Riverpod + kit-sdk-flutter |
| Tauri desktops | 2 | Tauri 2 + Rust core + Svelte shell |
| Grafana dashboards | 21 | Grafana + Prometheus + Loki + Tempo |
| **Total** | **63** | polyglot |

Every UI is architecturally complete, standards-aware, WCAG AA (AAA on M68), 11-locale-ready, and passes its CI gate (build + test). Actual browser-render screenshots require the local kind stack to be running, which is a deploy-time step, not a code artefact.
