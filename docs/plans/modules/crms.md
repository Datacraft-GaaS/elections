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

1. FR-M12-001: candidate self-nomination portal with document upload (nomination papers, IPRS, affidavit).
2. FR-M12-002: IPRS identity verification on candidacy submission.
3. FR-M12-003: statutory clearance workflow: KRA tax compliance, EACC integrity check, court clearance.
4. FR-M12-004: quota enforcement: gender (2/3 rule, Art.81b), youth, PWD quotas per constituency/region/national.
5. FR-M12-005: quota compliance dashboard for IEBC officers and parties.
6. FR-M12-006: candidate approval workflow: officer review -> legal sign-off -> public list.
7. FR-M12-007: candidate withdrawal: before deadline only; full audit; replacement workflow.
8. FR-M12-008: public candidate list published after nomination close; signed JSON + CSV.
9. FR-M12-009: ballot design package: signed PDF layout per position/constituency sent to gov printer.
10. FR-M12-010: ballot proof: HSM-signed hash of ballot design; publicly verifiable.
11. FR-M12-011: multi-position: simultaneous pipelines for all elective positions + referenda.
12. FR-M12-012: full audit trail: every status change signed + anchored.
13. FR-M12-013: RBAC: party agents, IEBC officers, legal team, printer liaison have scoped access.
14. FR-M12-014: deadline enforcement: submission, withdrawal, clearance windows enforced by OPA policy.
15. FR-M12-015: appeal workflow: rejected candidate can appeal; escalation to M40 election court interface.
16. Nomination deadline enforcement: OPA hard-close; no late submissions.
17. KRA/EACC circuit-breaker: fail-open on lookup; fail-closed on approval.
18. Ballot proof published to printer within 24 h of nomination close.
19. Public candidate list serves p95 <= 200 ms (CDN-cached).

## Acceptance criteria

AC-M12-01: quota compliance alerts fire when 2/3 rule is breached (UAT).
AC-M12-02: KRA circuit-breaker triggers on KRA blackhole; manual override available (chaos test).
AC-M12-03: ballot proof verifiable independently by reference client (integration test).
AC-M12-04: candidate withdrawal blocked after deadline (regression test).
AC-M12-05: all status changes in audit log; chain verifiable (audit trace).
AC-M12-06: appeal escalates to M40 correctly (integration test).
AC-M12-07: 99.9% availability during nomination cycle (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-08: DNS poisoned; DNSSEC; hardcoded fallback for KRA/EACC.

**time**
EC-TIM-09: nomination deadline OPA hard-close; all-or-nothing on in-flight.
EC-TIM-04: KRA/EACC SLA breach; manual clearance path.

**identity**
EC-IDN-02: JWT expires mid-upload; in-flight completes; next 401.
EC-IDN-10: SpiceDB down; fail closed quota-override writes.
EC-IDN-11: stale JWKS; refetch on kid miss.

**data**
EC-DAT-01: duplicate nomination same key; cached response.
EC-DAT-09: Unicode candidate names; NFC normalisation.
EC-DAT-13: withdrawn candidate still in public list (tombstone); status shown.

**security**
EC-SEC-05: injection on document upload; parameterised; content-type allowlist.
EC-SEC-06: SSRF via document URL; allowlist.
EC-SEC-16: secrets in logs refused.

**legal**
EC-LEG-01: candidate withdraws post-ballot-print; status frozen; ballot annotated.
EC-LEG-02: court injunction; status frozen by legal officer; audit-logged.

**human**
EC-HUM-06: screen readers; WCAG 2.2 AA.
EC-HUM-08: 11 languages.

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
