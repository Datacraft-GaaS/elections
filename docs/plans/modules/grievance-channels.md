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

Intake channels:

1. FR-M23-001: Public Verifier app intake: structured form with category selector, optional GPS station-code lookup, photo/video attachment compressed on-device, anonymous toggle; returns case-reference number and tracking URL with QR code.
2. FR-M23-002: USSD short-code intake: IVR menu in EN + SW; structured question tree; <= 5 menu steps; 182-char session limit; acknowledgement SMS with case reference within 60 s.
3. FR-M23-003: WhatsApp Business intake: verified IEBC WhatsApp number; natural-language intake with NLP category extraction via M28; category confirmation before case creation; returns case reference in-chat.
4. FR-M23-004: SMS intake: free-text to IEBC short-code; auto-categorised by rule and ML; acknowledgement SMS with case reference within 60 s.
5. FR-M23-005: Kit-side kiosk intake: locked mode on KIEMS kit at polling station; voter or party agent enters complaint category and optional voice note transcribed on-device via M28 ASR; Presiding Officer counter-signs receipt; case submitted to central on sync.
6. FR-M23-006: In-person IAB office intake: IEBC case officer records complaint directly in case management console; produces printed receipt for complainant.

Case intake and record:

7. FR-M23-007: All intake channels create a unified case record with: case_id (uuid7), intake_channel, category, severity (P0/P1/P2), station_code, constituency_code, reported_at, description, attachments (encrypted), contact_hash (SHA-256 of MSISDN/email; never plain).
8. FR-M23-008: Severity auto-classification: P0 (physical violence or threat at station, kit failure causing voter exclusion, ballot-box tampering, impersonation by official, double-voting in progress); P1 (bribery/undue influence, voter turned away without s.44A, party agent expelled, Form 34A not posted, queue > 300 minutes); P2 (voter information complaints, voter roll query, process confusion, general feedback).
9. FR-M23-009: Case officer assignment: P0 assigned immediately to on-duty senior officer via push notification; P1 to next-available officer in constituency queue; P2 to general queue.
10. FR-M23-010: SLA tracking: time-to-breach displayed per case; supervisor sees SLA-at-risk queue; auto-escalation to supervisor at 75% of SLA window with no case update.
11. FR-M23-011: P0 intake-to-officer-notification <= 2 minutes; timestamp delta measured in production and in integration tests.
12. FR-M23-012: Category routing: queue complaints forwarded to M19; speech/disinformation to M17; campaign-finance complaints to M22; fraud signals to M26; violence/security to ECC M16.
13. FR-M23-013: Duplicate grievance on same session and same body returns cached acknowledgement; no second case created; idempotency key derived from channel + body hash.
14. FR-M23-014: Anonymous submission on every channel: no account required; session token for case tracking only; complainant contact never mandatory.
15. FR-M23-015: Rate limiting: 50 submissions per hour per IP/MSISDN on public endpoints; rate-limit bypass attempts trigger monitoring alert.

Case management:

16. FR-M23-016: Case officer investigation tools: evidence upload, note-taking, status transitions, inter-officer hand-off with reason code, resolution with resolution code.
17. FR-M23-017: Case lifecycle states: OPEN, ASSIGNED, INVESTIGATING, PENDING_EVIDENCE, RESOLVED, REFERRED, CLOSED; every transition logged with actor ID and timestamp.
18. FR-M23-018: Case closure requires: resolution code, summary note, complainant notification via original intake channel, supervisor sign-off for P0 and P1.
19. FR-M23-019: Complainant status tracking: query case status via any intake channel using case reference; returns status and last-updated timestamp; no PII in status response.

Whistleblower protection:

20. FR-M23-020: Protected whistleblower channel: submissions marked WHISTLEBLOWER get a privilege flag; case file visible only to IEBC Supervisor, IEBC Commissioner, and IAB legal representative; no other case officer can access the underlying identity even if contact was provided.
21. FR-M23-021: Whistleblower contact stored as SHA-256 hash of MSISDN/email only; plain text never written to any store, log, or trace; legal-privilege flag prevents standard discovery.
22. FR-M23-022: Whistleblower decryption ceremony: dual-officer approval required to access contact data; every decryption access event logged and anchored to Merkle chain.
23. FR-M23-023: Whistleblower protection scope confirmed under Whistleblower Protection Act 2022; legal sign-off required before feature is enabled in production.

Evidence packages:

24. FR-M23-024: Evidence-package generation for DCI/ODPP referral: for cases under EOA 2016 ss.6-12, assembles HSM-signed PDF + JSON package containing: case record, encrypted attachments, station telemetry extract, kit audit log extract, officer investigation notes, statutory basis, timestamp chain, Merkle inclusion proof.
25. FR-M23-025: 100% of verified EOA ss.6-12 cases referred to DCI/ODPP endpoint within SLA; referral logged in audit trail.
26. FR-M23-026: Evidence-package generation also covers general election offences under Elections Act ss.83-84; statutory basis field set to the specific provision.
27. FR-M23-027: Evidence package assembled within 30 minutes of officer request; HSM signing required before dispatch; no unsigned package routed.
28. FR-M23-028: Every evidence package carries a Merkle inclusion proof verifiable by the receiving party without access to the internal system.
29. FR-M23-029: Evidence-package chain of custody auditable per Evidence Act Cap 80 and Electronic Transactions Act 2007; ODPP format co-designed with legal team and validated at T-30 dry run.

Public transparency:

30. FR-M23-030: Grievance heat map: public-facing anonymised choropleth showing case volume per constituency by severity tier; updated every 15 minutes; disclaimer states "cases are unverified reports."
31. FR-M23-031: Post-election case analytics report: total cases by category, channel, severity, SLA compliance rate, resolution code, and ODPP referrals; published within T+30 of election day.
32. FR-M23-032: Observer read-only anonymised grievance feed API: KEOG/ELOG/EU EOM access via JWT with SpiceDB-enforced read-only scope; no PII returned.

APIs:

33. FR-M23-033: POST /v1/grievances: public API; rate-limited 50/hr per IP; idempotent write; anonymous allowed; returns case_id.
34. FR-M23-034: GET /v1/grievances/{id}/status: public API; rate-limited 100/hr; status and last-updated only; no PII in response.
35. FR-M23-035: PATCH /v1/grievances/{id}: COMMISSION_OFFICER JWT + SpiceDB; idempotent write; case status, note, assignment.
36. FR-M23-036: POST /v1/grievances/{id}/evidence-package: COMMISSION_OFFICER JWT + SpiceDB; idempotent write; assembles package.
37. FR-M23-037: POST /v1/grievances/{id}/evidence-package/{pkg_id}/sign: COMMISSION_SUPERVISOR JWT + HSM; rate-limited 5/hr; signs and routes package to DCI/ODPP.
38. FR-M23-038: GET /v1/grievances/heat-map: public; rate-limited 500/min; constituency-level case volume only.
39. FR-M23-039: GET /v1/grievances/analytics: COMMISSION_ADMIN JWT + SpiceDB; full analytics dataset including SLA compliance breakdown.
40. FR-M23-040: Webhook intake endpoints: POST /v1/grievances/intake/whatsapp, /sms, /ussd; authenticated with WEBHOOK_SECRET; rate-limited 1,000/min each; verified before case creation.
41. FR-M23-041: Webhook dispatcher: accredited observers and party agents subscribe to case lifecycle events via POST /grievance/v1/webhooks; events HMAC-signed; delivered at-least-once idempotent.

Events and audit:

42. FR-M23-042: Every case action signed with actor ID, action type, and HLC timestamp; appended to Merkle chain within 60 minutes or on sync if offline.
43. FR-M23-043: Kafka events emitted: eel.grievance.case_created.v1, eel.grievance.case_escalated.v1, eel.grievance.case_resolved.v1, eel.grievance.evidence_assembled.v1 (HSM signed, exactly-once), eel.grievance.whistleblower_registered.v1 (HSM encrypted, exactly-once).

Compliance and data protection:

44. FR-M23-044: DPA 2019 lawful basis: complaint data processed under s.30(1)(c) (vital interests) and s.30(1)(b) (legal obligation for evidence packages); complainant contact encrypted at rest; consent obtained for follow-up communications.
45. FR-M23-045: 7-year retention of case records post-election; DPIA registered with ODPC before system goes live.

Availability and scale:

46. FR-M23-046: All intake channels available >= 99.9% during T-7 to T+7 election week; measured per channel independently.
47. FR-M23-047: System handles 50,000 concurrent incoming cases on election day without degradation; p99 response <= 500 ms.
48. FR-M23-048: Kit-side kiosk stores cases locally (AES-256 encrypted); syncs to central within 60 minutes of connectivity restore; no case dropped on queue overflow.
49. FR-M23-049: WA quota exhaustion triggers automatic fallback to USSD/SMS; no case dropped; ECC alerted when WA quota drops below 20%; USSD/SMS carry full intake volume independently.

Accessibility and language:

50. FR-M23-050: WCAG 2.2 AA on web and kiosk surfaces; screen-reader compatible; keyboard-only navigable; independent audit before go-live.
51. FR-M23-051: Multi-language intake: EN + SW on all channels; WhatsApp and IVR additionally offer Kikuyu, Luo, Kamba, Kalenjin, and Luhya; architecture extensible to all 11 official languages.
52. FR-M23-052: USSD session constrained to 182-character budget per step; all prompts tested at 150% text expansion across supported locales.

UI screens:

53. FR-M23-053: Public intake form (Verifier App + Web): category selector with icons, station-code lookup by GPS, photo/video attach, anonymous toggle; submit returns case reference with QR code.
54. FR-M23-054: Case status tracker: enter case reference; status timeline (submitted, assigned, investigating, resolved); estimated resolution time per severity; contact IEBC link; no PII shown.
55. FR-M23-055: IEBC case management console: kanban-style OPEN/ASSIGNED/INVESTIGATING/RESOLVED columns; SLA countdown badges; severity colour coding; one-click assignment and hand-off; evidence-package builder embedded.
56. FR-M23-056: P0 alert panel: dedicated supervisor view; live count of P0 cases; time-to-SLA-breach per case; push notification log; one-click reassignment to available officer.
57. FR-M23-057: Evidence package builder screen: case summary, auto-selected attachments, station telemetry extract, statutory basis picker, PDF preview, HSM-sign button; signing disabled until statutory basis selected.
58. FR-M23-058: Kit kiosk intake screen: simplified 4-step flow (category icon grid, description voice/text, Presiding Officer counter-sign, printed receipt with case reference); no network required for intake.

## Acceptance criteria

AC-M23-01: P0 intake-to-officer-notification <= 2 minutes; timestamp delta verified in integration test; 99th percentile measured over T-7 to T+7 load scenario.
AC-M23-02: P0 SLA compliance rate >= 99%; SLA tracking log verified; IEBC Commissioner sign-off required before production go-live.
AC-M23-03: P1 SLA compliance rate >= 95% over election week; SLA tracking log.
AC-M23-04: P2 SLA compliance rate >= 90% over election week; SLA tracking log.
AC-M23-05: USSD intake completes case submission in <= 5 menu steps and <= 182-char per step; UAT walkthrough with three independent testers.
AC-M23-06: Whistleblower contact accessible only with dual-officer approval logged and anchored; security test confirms no single-officer path exists; audit log shows every decryption event.
AC-M23-07: Evidence package Merkle inclusion proof verifiable by receiving party without system access; integration test confirms proof for 100% of assembled packages.
AC-M23-08: GET /v1/grievances/{id}/status returns zero PII fields in any response path; automated scanner and manual security review pass.
AC-M23-09: WA quota exhausted; USSD/SMS fallback initiated automatically with no case dropped; chaos test confirms at >= 10,000 simultaneous in-flight cases.
AC-M23-10: WCAG 2.2 AA on web portal and kiosk intake screen; independent audit pass; screen-reader full-task-completion verified.
AC-M23-11: 99.5% availability measured over 30-day rolling window; SLO burn-rate alert fires before budget exhausted; election-week target 99.9% per channel.
AC-M23-12: Grievance heat map updates within 15 minutes of new case intake; latency measurement across 100 consecutive update cycles.
AC-M23-13: Evidence package delivered to DCI/ODPP within 6 hours of P0 verification for 100% of verified EOA ss.6-12 cases; referral log audited at T+7.
AC-M23-14: System handles 50,000 concurrent case submissions; p99 response time <= 500 ms; load test result attached to release note.
AC-M23-15: All case action events anchored to Merkle chain within 60 minutes; Merkle reconciliation script confirms 100% coverage at T+1.

## Applicable edge cases

EC-CON-02: 3G/500 kbps link on election day; USSD/SMS serve as primary intake; web portal served from CDN edge; module 3G SLAs met.
EC-CON-03: 2G only; kit refuses data path; USSD/SMS always available as fallback intake; case reference delivered via SMS reply.
EC-CON-05: Flapping link during grievance submission; idempotent write with body-hash key; cached acknowledgement returned; no duplicate case created.
EC-CON-11: Kafka ISR shrink; case-created event accumulates in outbox; drained when brokers recover; no event lost.
EC-DAT-01: Duplicate grievance same session and same body; cached acknowledgement returned; no second case record created.
EC-DAT-02: Same idempotency key with different grievance body; 422 idempotency-key-reuse error returned; no override of existing case.
EC-DAT-09: Unicode grievance text with combining characters or right-to-left script; NFC normalisation applied before storage and search.
EC-DAT-14: Evidence attachment > 1 MB; outbox rejects; blob stored in MinIO with hash reference in case record; no truncation.
EC-DAT-16: Legacy USSD payload with shortened free-text message; parser normalises to structured case record with default category for officer review.
EC-SEC-05: SQL or NoSQL injection in grievance text or description field; parameterised storage only; no dynamic query evaluation.
EC-SEC-06: SSRF via evidence upload URL submitted by reporter; allowlist of permitted domains enforced; private-range and rebinding attempts blocked.
EC-SEC-09: Rate-limit evasion on grievance submission endpoint; key on IP and MSISDN; priority shed on burst; monitoring alert on pattern.
EC-SEC-16: Whistleblower contact hash or evidence path leaked to log lines; CI log-scanner fixture blocks deployment; telemetry refuses Tier A/B data.
EC-IDN-02: JWT expires mid case-officer action; in-flight request completes; next request returns 401 with silent token refresh.
EC-IDN-04: Anonymous submission on feature phone; no attestation required; USSD/SMS path always available without app install.
EC-IDN-08: Duress signal in grievance text (keyword or phrase); auto-classified P0; covert ECC alert triggered without alarming reporter.
EC-IDN-10: SpiceDB down; fail closed on triage-write and officer-assign writes; intake channels still accept and queue; cached class-C reads continue.
EC-SCL-01: 5M portal visitors per hour on election day; >= 95% served from CDN edge; origin p99 <= 500 ms; heat-map served from CDN cache.
EC-SCL-04: Kafka consumer lag > 60 s on case-created topic; page alert triggered; backpressure applied; no OOM on worker.
EC-OPS-01: Kit battery death mid-kiosk intake session; journal-before-act pattern; exact resume at same complaint step; no double case submission.
EC-OPS-07: Case officer absent or credential revoked mid-investigation; case reassigned by supervisor; old credential revoked exactly once; new assignment logged.
EC-LEG-08: Evidence package requested by DCI at T+6d 23 h; 24-hour clock not blocked by freeze; bundle delivered without hold.
EC-LEG-09: Duplicate EOA ss.6-12 auto-referral race condition; single referral ensured via idempotent scheduler with case_id as idempotency key.
EC-HUM-06: Screen-reader users access grievance intake form; WCAG 2.2 AA; full task completion verified including file attach and submission.
EC-HUM-08: 11 language strings including Kikuyu, Luo, Kamba; USSD 182-char budget tested per locale at 150% expansion.
EC-HUM-10: Low-literacy USSD path; numeric menu <= 5 steps; IVR audio alternative for each step; kiosk uses icon grid.

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
