# M12 crms

| field | value |
|---|---|
| M-number | M12 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M12-crms.md |

## Purpose

Candidate and party Registration Management System: capture nomination submissions, verify statutory clearances (KRA, EACC, court), enforce gender/youth/PWD quota rules, and produce cryptographically signed ballot design packages for the government printer.

## Surfaces and languages

| surface | technology | path under services/crms-svc/ |
|---|---|---|
| Party/candidate portal | Astro on packages/eel-web-astro | astro/crms-portal/ |
| Nomination API | Rust (axum + utoipa) | crates/crms-api/ |
| Domain + quota logic | Rust | crates/crms-domain/ |
| Admin back-office | Django on packages/eel-web-py | django/crms-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Candidate | candidate_id, id_number_hash, name, party_id, position_code, constituency_code, status, gender, dob | B | T2 |
| Nomination | nom_id, candidate_id, submitted_at, documents_refs, statutory_status, cleared_at | C | T2 |
| ClearanceDecision | decision_id, nom_id, source (KRA/EACC/court), verdict, evidence_ref, decided_at | C | T2 |
| BallotProof | proof_id, election_id, position_code, ballot_hash, printer_sig, approved_at | D | T1-audit |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/crms/nomination.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /crms/v1/candidates | 2 live-per-user | JWT | yes | Submit candidacy; triggers IPRS verify |
| GET /crms/v1/candidates/{id} | 2 live-per-user | JWT | n/a | Candidate status |
| POST /crms/v1/nominations/{id}/documents | 2 live-per-user | JWT | yes | Upload statutory documents |
| POST /crms/v1/clearances | 3 accredited | JWT+SpiceDB | yes | Record clearance decision from KRA/EACC/court |
| GET /crms/v1/candidates | 1 signed-static | public | n/a | Public candidate list (after nomination close) |
| GET /crms/v1/quotas/{scope}/{code} | 3 accredited | JWT+SpiceDB | n/a | Quota compliance status per constituency/region |
| POST /crms/v1/ballot-proofs | 3 accredited | JWT+SpiceDB | yes | Publish signed ballot design package |
| GET /crms/v1/ballot-proofs/{id} | 1 signed-static | public | n/a | Ballot proof (hash + printer sig) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.crms.candidate_submitted.v1 | emits | eel.crms.candidate.v1 | operational | yes | at-least-once idempotent |
| eel.crms.clearance_received.v1 | emits | eel.crms.candidate.v1 | operational | yes | at-least-once idempotent |
| eel.crms.candidate_approved.v1 | emits | eel.crms.candidate.v1 | operational | yes | at-least-once idempotent |
| eel.crms.ballot_proof_published.v1 | emits | eel.crms.ballot.v1 | operational | yes (HSM) | exactly-once |
| eel.bvr.voter_status_changed.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| IPRS | I01 | integrations/iprs/ | yes | fail-open on read; fail-closed on approval |
| KRA iTax | I12 | integrations/kra/ | yes | circuit-breaker; IEBC officer manual override |
| EACC | external | integrations/eacc/ | yes | clearance queued; async response |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on ballot proof publish |
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | voter identity verify |
| Government printer | external | integrations/gov-printer/ | yes | ballot-proof push; retry |

## ML and AI

none

## Dependencies

- Modules: M02 (voter identity check), M22 (campaign finance references candidate IDs), M47 (nomination fee payment)
- Platform: P04 (IAM), P05 (audit), P06 (HSM for ballot proof)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro, eel-web-py

## Feature checklist

Registration and nomination:
1. FR-M12-001: party portal MFA login per party official; FIDO2 or TOTP; session scoped to party. (spec FR-M12-001)
2. FR-M12-002: nomination form per elective position: biographic, ICAO-compliant portrait, party affiliation, running-mate for President/Governor, area code, symbol. (spec FR-M12-002)
3. FR-M12-003: independent-candidate registration with signature-collection evidence upload; evidence integrity check. (spec FR-M12-003)
4. FR-M12-004: voter-register cross-check via M02: candidate must be a registered voter; automatic verification at submission. (spec FR-M12-004)
5. FR-M12-005: KRA tax-compliance check via adapter (advisory); circuit-breaker on KRA timeout; manual override with reason logged. (spec FR-M12-005)
6. FR-M12-006: EACC integrity clearance check via adapter (advisory); similar circuit-breaker and override flow. (spec FR-M12-006)
7. FR-M12-007: document upload: educational certificates, integrity clearance, ethics affidavits, party clearance; PDF only; malware scan; max 20 MB per document. (spec FR-M12-006)
8. FR-M12-008: gender quota enforcement per Elections Act (two-thirds gender rule) at party and national level; alert on violation. (spec FR-M12-007)
9. FR-M12-009: PWD and youth quota tracking: counts per party per position; dashboard for IEBC officers and parties. (spec FR-M12-007)
10. FR-M12-010: nominations window enforcement: OPA hard-close at gazetted deadline; no further submissions accepted. (spec FR-M12-013)

Clearance workflow:
11. FR-M12-011: RO clearance workflow: review submission, request additional documents, approve or reject with reason. (spec FR-M12-008)
12. FR-M12-012: clearance lifecycle: SUBMITTED -> DOCS_UPLOADED -> RO_REVIEWING -> CLEARED / REJECTED -> GAZETTED. (spec FR-M12-009)
13. FR-M12-013: candidate appeal workflow: rejected candidate appeals; escalation to M40 election court interface. (spec FR-M12-015)
14. FR-M12-014: withdrawal workflow: before deadline only; full audit trail; replacement nomination workflow. (spec FR-M12-007)
15. FR-M12-015: dual-officer sign-off required for gazette publication step; separate HSM challenges. (spec FR-M12-010)

Publication and ballot:
16. FR-M12-016: public gazette publication of cleared candidates with all statutory details; signed JSON and CSV. (spec FR-M12-010, FR-M12-008)
17. FR-M12-017: ballot proof generation: PDF layout per position matching printer specifications; symbol plus portrait plus name plus party colour. (spec FR-M12-011)
18. FR-M12-018: ballot lottery or registration-time ordering per IEBC policy; deterministic and reproducible. (spec FR-M12-011)
19. FR-M12-019: ballot proof preview and party sign-off required before printer handoff. (spec FR-M12-012)
20. FR-M12-020: HSM-signed hash of ballot PDF published for public verifiability. (spec FR-M12-010)
21. FR-M12-021: ballot proof delivered to printer within 24 h of nomination close.
22. FR-M12-022: multi-position: simultaneous independent pipelines for all elective positions and referenda. (spec FR-M12-011)

Audit, access control, and monitoring:
23. FR-M12-023: full audit trail: every status change signed and anchored to transparency log. (spec FR-M12-014)
24. FR-M12-024: RBAC: party agents, IEBC officers, legal team, and printer liaison have scoped access; SpiceDB enforced. (spec FR-M12-013)
25. FR-M12-025: public candidate list CDN-cached; p95 <= 200 ms; no PII beyond statutory fields.
26. FR-M12-026: candidate withdrawal post-ballot-print: status frozen on ballot; tallied and annotated per EC-LEG-01. (spec FR-M12-007)
27. FR-M12-027: WCAG 2.2 AA on public gazette and party portal; screen-reader and keyboard-only support. (spec NFR-M12-003)
28. FR-M12-028: multilingual: English and Kiswahili on all surfaces; extensible to 11 languages.
29. FR-M12-029: SSRF prevention on document and URL fields in nomination form; allowlist; no private IP ranges.
30. FR-M12-030: XXE and unsafe deserialization prevention on PDF and SAML inputs; DTD off; strict models.
31. FR-M12-031: secrets in logs refused; Semgrep Tier A/B log ban enforced in CI.
32. FR-M12-032: injection prevention on all nomination and search inputs; parameterised queries; ZAP scan clean.

## Acceptance criteria

AC-M12-01: end-to-end nomination + clearance + gazette flow passes UAT with real party and IEBC personas.
AC-M12-02: voter-register cross-check prevents non-voter candidates at submission (bench).
AC-M12-03: gender quota alert fires when two-thirds rule is breached (UAT).
AC-M12-04: ballot proof matches printer specification; party sign-off captured (printer sign-off).
AC-M12-05: nominations window OPA hard-close enforced; no late submissions accepted (regression test).
AC-M12-06: public gazette accessible with WCAG 2.2 AA audit pass (independent audit).
AC-M12-07: appeal escalation to M40 court interface works end-to-end (integration test).
AC-M12-08: KRA/EACC circuit-breaker triggers on blackhole; manual override path available (chaos test).
AC-M12-09: all status changes in audit log; chain verifiable against transparency log (audit trace).
AC-M12-10: 99.9% availability during nomination cycle (SLO measurement).
AC-M12-11: 11-language smoke test: no truncated strings on gazette and portal (i18n test).
AC-M12-12: ballot proof HSM signature verifiable independently by reference client (integration test).

## Applicable edge cases

connectivity:
EC-CON-05: flapping link; nomination draft saved locally; retry on reconnect; no duplicate submission.
EC-CON-08: DNS poisoned; DNSSEC; hardcoded fallback endpoints for KRA and EACC adapters.

time:
EC-TIM-04: DST/TZ: nomination deadline computed in UTC; displayed in EAT; no off-by-one at boundary.
EC-TIM-09: nomination deadline OPA hard-close; all-or-nothing on in-flight submission at deadline second.

identity:
EC-IDN-02: JWT expires mid-document upload; in-flight upload completes; next request 401.
EC-IDN-09: dual-officer gazette publication: re-validate both at commit; approver revocation mid-flow causes re-delegation.
EC-IDN-10: SpiceDB down; quota-override and gazette writes fail closed; read-only portal continues.
EC-IDN-11: stale JWKS; refetch on kid miss; no 401 storm on party portal.

data:
EC-DAT-01: duplicate nomination same key; cached response; no re-processing.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-09: Unicode candidate names; NFC normalisation; grapheme-safe display on gazette.
EC-DAT-13: withdrawn candidate still in public gazette (tombstone); withdrawn status shown clearly.

security:
EC-SEC-05: injection on nomination form inputs and document metadata; parameterised only.
EC-SEC-06: SSRF via document URL fields; allowlist; no private IP ranges; rebinding guard.
EC-SEC-08: XXE on PDF and SAML inputs; DTD off; no pickle; strict Pydantic models.
EC-SEC-16: secrets in logs refused; party credential hashes not in plaintext telemetry.

legal:
EC-LEG-01: candidate withdraws post-ballot-print; status frozen; ballot annotated; tallied separately.
EC-LEG-02: court-ordered postponement or injunction; status frozen by legal officer; audit-logged.

human:
EC-HUM-06: screen readers; WCAG 2.2 AA on gazette and party portal.
EC-HUM-08: 11 languages; no string truncation; ICU plural forms.

scale:
EC-SCL-04: consumer lag > 60 s on clearance event topic; backpressure applied; no OOM.
EC-SCL-08: Valkey eviction of idempotency keys for document upload; PG-durable store used.

EC-OPS-04: degradation-mode misconfiguration; OPA policy rejects nomination writes; gazette reads continue.
EC-OPS-09: wrong degradation mode; canary telemetry detects; auto-rollback triggered.
EC-DAT-07: partial document upload; atomic store; no partial record visible to clearance workflow.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | quota logic, clearance FSM, deadline OPA policy, ballot-proof assembly | nextest | M12-UNIT- |
| property | quota invariant: never approve if 2/3 violated; deadline always enforced | proptest | M12-PROP- |
| contract | IPRS adapter (consumer); KRA adapter (consumer); M40 appeal event (provider) | Pact | M12-PACT- |
| integration | IPRS double; KRA double; EACC double; SoftHSM ballot proof | nextest + testcontainers | M12-INT- |
| e2e | candidate submit -> clearances -> approve -> ballot proof -> printer push | Playwright | M12-E2E- |
| fuzz | nomination form inputs; document upload; ballot design JSON | cargo-fuzz | M12-FUZZ- |
| load | nomination-day peak; quota recalculation at scale | k6 | M12-LOAD- |
| security | OWASP ZAP; Semgrep; injection; SSRF on doc upload | ZAP + Semgrep | M12-SEC- |
| a11y | WCAG 2.2 AA; keyboard-only; screen-reader | axe-core + manual | M12-A11Y- |
| i18n | EN + SW; 11 locales | i18n suite | M12-I18N- |

## Runbooks and dashboards

- docs/runbooks/crms-svc-degradation.md (create): KRA/EACC outage; IPRS circuit open; ballot proof HSM fail; nomination deadline emergency extension.
- platform/observability/crms-svc/grafana-nominations.json: submission rate, clearance queue depth, quota compliance status, ballot proof publish latency.
- platform/observability/crms-svc/slo-alerts.yaml: 99.9% burn rate; clearance queue > 72 h; quota violation unresolved.

## Open questions

- KRA/EACC API availability: both external APIs have historically been unreliable; SLA and retry policy need formal MoU before nomination cycle.
- Printer PDF fidelity: the ballot design PDF format must be validated against the government printer's pre-press system before G4 pilot.
