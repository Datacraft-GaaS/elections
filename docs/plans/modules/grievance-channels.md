# M23 grievance-channels

| field | value |
|---|---|
| M-number | M23 |
| Home service | ecc-svc (ADR-0020) |
| Owning work package | WP-4.1 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M23-grievance-channels.md |

## Purpose

Multi-channel grievance intake for citizens, observers, and whistleblowers: accept reports of electoral irregularities via WhatsApp, USSD, SMS, mobile app, kiosk, and API; triage by severity and category; route to ECC, speech integrity, queue transparency, or campaign finance as appropriate; maintain a whistleblower protection profile; and generate evidence packages for legal proceedings.

## Surfaces and languages

| surface | technology | path under services/ecc-svc/ |
|---|---|---|
| Public web + kiosk intake | Astro on packages/eel-web-astro | astro/grievance-portal/ |
| WhatsApp bot | Rust (axum + WA Cloud API I11) | crates/grievance-wa/ |
| USSD / SMS handler | Rust | crates/grievance-ussd/ |
| Mobile app intake | Flutter on packages/kit-sdk-flutter (shared with M13) | apps/citizen-verify/ |
| Grievance API | Rust (axum + utoipa) | crates/grievance-api/ |
| Case management admin | Django on packages/eel-web-py | django/grievance-admin/ |
| Webhook dispatcher | Rust | crates/grievance-webhooks/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| GrievanceCase | case_id, channel, category, severity, description_hash, evidence_refs, status, assigned_to, resolved_at | C | T2 |
| EvidencePackage | pkg_id, case_id, content_refs, chain_of_custody, sig, assembled_at | C | T1-audit |
| WhistleblowerProfile | profile_id, case_id, contact_encrypted, pseudonym, protection_level, created_at | A | T1-voter (encrypted, access-controlled) |
| WebhookSubscription | sub_id, subscriber_id, event_types, endpoint_url, secret_hash | C | T2 |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/ecc/grievance.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /grievance/v1/cases | 2 live-per-user | public | yes | Submit grievance (any channel); anonymous allowed |
| GET /grievance/v1/cases/{id}/status | 2 live-per-user | public | n/a | Case status by reference; no PII returned |
| POST /grievance/v1/cases/{id}/evidence | 2 live-per-user | public | yes | Upload additional evidence |
| GET /grievance/v1/cases | 3 accredited | JWT+SpiceDB | n/a | Case queue for IEBC triage |
| PATCH /grievance/v1/cases/{id} | 3 accredited | JWT+SpiceDB | yes | Triage: assign, escalate, resolve |
| POST /grievance/v1/evidence/{case_id}/package | 3 accredited | JWT+SpiceDB | yes | Assemble signed evidence package |
| POST /grievance/v1/webhooks | 3 accredited | JWT+SpiceDB | yes | Register webhook subscription |
| DELETE /grievance/v1/webhooks/{id} | 3 accredited | JWT+SpiceDB | yes | Remove webhook subscription |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.grievance.case_created.v1 | emits | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.case_escalated.v1 | emits | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.case_resolved.v1 | emits | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.evidence_assembled.v1 | emits | eel.grievance.evidence.v1 | operational | yes (HSM) | exactly-once |
| eel.grievance.whistleblower_registered.v1 | emits | eel.grievance.wb.v1 | event-source | yes (HSM encrypted) | exactly-once |
| eel.rts.form_captured.v1 | consumes | eel.rts.form.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| WhatsApp Cloud API | I11 | integrations/whatsapp/ | yes (stub) | WA quota; 6-8 wk approval; USSD fallback |
| USSD/SMS gateway | I08 | integrations/sms-gw/ | yes | SMS fallback; retry |
| ECC M16 | internal | integrations/ecc/ | yes | severity-escalated cases; circuit breaker |
| Speech integrity M17 | internal | integrations/speech/ | yes | grievance text forwarded for content classification |
| Queue transparency M19 | internal | integrations/queue/ | yes | queue-related grievances forwarded as signal |
| Campaign finance M22 | internal | integrations/finance/ | yes | finance complaints forwarded |
| Anomaly/fraud M26 | internal | integrations/anomaly/ | yes | grievance signal to fraud detection |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on evidence package + whistleblower encrypt |
| Transparency log P05 | internal | integrations/tlog/ | yes | evidence anchoring queued |

## ML and AI

none in this module; content classification delegated to M17 speech integrity.

## Dependencies

- Modules: M08, M10 (form context for form-related grievances), M13 (intake relay), M16 (escalation target), M17 (content screening), M19 (queue signal), M22 (finance signal), M26 (fraud signal)
- Platform: P04 (IAM), P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro, eel-web-py, kit-sdk-flutter

## Feature checklist

1. FR-M23-001: anonymous grievance submission on all channels; no account required.
2. FR-M23-002: WhatsApp bot intake: natural-language triage in EN + SW; guided form on request.
3. FR-M23-003: USSD intake: <= 5 steps; 182-char limit; returns case reference.
4. FR-M23-004: SMS intake: free-text to dedicated short code; case reference via reply.
5. FR-M23-005: web/kiosk intake: file + photo + audio evidence upload.
6. FR-M23-006: Flutter app intake: shared with M13; grievance section.
7. FR-M23-007: whistleblower protection: encrypted contact; pseudonym; protection level classification.
8. FR-M23-008: severity triage: critical (violence/fraud) auto-escalate to ECC within 5 min; high within 1 h; medium within 4 h.
9. FR-M23-009: category routing: queue complaints to M19; speech/disinformation to M17; finance to M22; fraud to M26; security to ECC M16.
10. FR-M23-010: case status portal: citizen tracks case by reference; no PII exposed.
11. FR-M23-011: evidence package assembly: HSM-signed; chain of custody; sent to IEBC legal or court.
12. FR-M23-012: webhook dispatcher: accredited observers/parties subscribe to case events.
13. FR-M23-013: WCAG 2.2 AA on web and kiosk surfaces.
14. FR-M23-014: multilingual: English + Kiswahili; extensible to 11 languages.
15. FR-M23-015: DPA compliance: whistleblower contact data encrypted at rest; access dual-controlled.
16. FR-M23-016: audit trail: every case action signed and anchored.
17. Critical grievance auto-escalate to ECC within 5 min.
18. WA response p95 <= 30 s; USSD session p95 <= 5 s; SMS ack <= 60 s.
19. Evidence package assembled within 30 min of request.
20. Whistleblower contact data accessible only with dual-officer approval.

## Acceptance criteria

AC-M23-01: critical grievance escalates to ECC within 5 min (integration test).
AC-M23-02: USSD intake completes case submission in <= 5 steps (UAT).
AC-M23-03: whistleblower contact accessible only with dual-officer approval (security test).
AC-M23-04: evidence package chain of custody verifiable (integration test).
AC-M23-05: case status returns reference without PII (security review).
AC-M23-06: WA quota exhausted; USSD fallback initiated (chaos test).
AC-M23-07: WCAG 2.2 AA on web portal (independent audit).
AC-M23-08: 99.5% availability (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-02: mobile-first; CDN portal; USSD/SMS always available.
EC-CON-03: 2G only; USSD/SMS primary; web fallback.

**identity**
EC-IDN-04: anonymous submission; no account required; session token for case tracking only.
EC-IDN-08: duress signal in grievance text; auto-classify as critical; ECC alert.
EC-IDN-10: SpiceDB down; fail closed triage-write; intake still accepts.

**data**
EC-DAT-01: duplicate grievance same session + same body; cached ack.
EC-DAT-09: Unicode grievance text; NFC normalisation.
EC-DAT-16: legacy USSD payload (short message); parser normalises.

**security**
EC-SEC-05: injection in grievance text; parameterised storage; no eval.
EC-SEC-06: SSRF via evidence upload URL; allowlist only.
EC-SEC-09: rate-limit per channel per IP; anti-spam; DDoS shedding.
EC-SEC-16: secrets/PII in log lines refused.
EC-SEC-18: whistleblower contact encrypted with per-record key; dual-controlled decryption.

**legal**
EC-LEG-12: whistleblower protection law: Kenya Whistleblower Protection Act; legal must confirm protection scope.
EC-LEG-05: evidence package for court: chain of custody auditable; format accepted by eCourt system.

**human**
EC-HUM-06: screen readers; WCAG 2.2 AA.
EC-HUM-08: 11 languages; USSD 182-char.
EC-HUM-10: low-literacy; USSD guided flow; icons on kiosk.

**scale**
EC-SCL-09: WA quota exhausted; fallback to USSD/SMS; no case dropped.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | triage FSM, severity classifier, category router, whistleblower encrypt, evidence builder | nextest + Flutter test | M23-UNIT- |
| property | critical case always escalates within 5 min; whistleblower contact always encrypted | proptest | M23-PROP- |
| contract | M13 intake relay (consumer); M16 escalation (provider); M17 speech relay (provider); M19 queue signal (provider); M22 finance signal (provider) | Pact | M23-PACT- |
| integration | WA stub; USSD double; SoftHSM evidence sign; tlog anchor; dual-officer whistleblower decrypt | nextest + testcontainers | M23-INT- |
| e2e | WA -> case -> triage -> ECC escalation -> evidence package; USSD case -> status track | ExUnit + Playwright | M23-E2E- |
| fuzz | grievance text; USSD input; evidence upload; webhook payload | cargo-fuzz | M23-FUZZ- |
| load | election-day grievance spike; WA quota near-limit; USSD concurrency | k6 | M23-LOAD- |
| chaos | WA quota hit; USSD gateway down; HSM kill; ECC disconnect | Chaos Mesh | M23-CHAOS- |
| security | whistleblower PII exfil; SSRF evidence upload; injection; rate-limit bypass | ZAP + Semgrep | M23-SEC- |
| a11y | WCAG 2.2 AA; screen-reader; keyboard-only; kiosk touch | axe-core + manual | M23-A11Y- |
| i18n | EN + SW; 11 locales; USSD 182-char budget test | i18n suite | M23-I18N- |

## Runbooks and dashboards

- docs/runbooks/grievance-channels-degradation.md (create): WA quota exhausted; USSD gateway down; HSM fail; critical case triage backlog.
- docs/runbooks/grievance-whistleblower-access.md (create): dual-officer whistleblower contact decryption ceremony.
- platform/observability/ecc-svc/grafana-grievance.json: case intake rate by channel, escalation latency, triage backlog, WA quota remaining, evidence assembly time.
- platform/observability/ecc-svc/slo-alerts.yaml: 99.5% burn rate; critical case escalation > 5 min; WA quota < 20%; triage backlog > 100 cases.

## Open questions

- WhatsApp approval: WA Cloud API business verification takes 6-8 weeks; application must start at T-12 months; delay pushes WA channel out of scope for the election cycle.
- Whistleblower Protection Act scope: the Kenya Whistleblower Protection Act 2017 protections apply to public sector; IEBC legal must confirm coverage for election observers and citizens before the whistleblower feature is enabled.
