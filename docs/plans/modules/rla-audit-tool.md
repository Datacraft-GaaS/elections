# M21 rla-audit-tool

| field | value |
|---|---|
| M-number | M21 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M21-rla-audit-tool.md |

## Purpose

Risk-Limiting Audit (RLA) tool: drive post-election statistical sampling of paper ballots against the digital tally using ballot-polling or comparison audit methods; generate NIST-compliant audit runs with cryptographically verifiable entropy sources; produce auditor reports and recount-trigger recommendations for IEBC and the courts.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Auditor desktop app (offline-capable) | Tauri (Rust shell + Astro frontend) | apps/rla-desktop/ |
| Web audit interface | Astro on packages/eel-web-astro | astro/rla-portal/ |
| RLA API | Rust (axum + utoipa) | crates/rla-api/ |
| Audit engine | Rust | crates/rla-engine/ |
| CLI tools | Rust (clap) | crates/rla-cli/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AuditRun | run_id, election_id, position_code, audit_method, risk_limit, seed_ref, status, completed_at, sig | D | T1-audit |
| SampledBallot | ballot_id, run_id, station_code, batch_id, ballot_position, auditor_interpretation, discrepancy | C | T1-audit |
| EntropySource | source_id, run_id, type (dice/beacon/hardware), value_hash, witnessed_at, witness_sigs | D | T1-audit |
| AuditReport | report_id, run_id, conclusion (pass/escalate), risk_achieved, discrepancy_count, sig, published_at | D | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/governance/rla.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /rla/v1/runs | 3 accredited | JWT+SpiceDB | yes | Start audit run; requires entropy input |
| GET /rla/v1/runs/{id} | 3 accredited | JWT+SpiceDB | n/a | Audit run status + sample list |
| POST /rla/v1/runs/{id}/ballots/{bid} | 3 accredited | JWT+SpiceDB | yes | Submit auditor interpretation for a ballot |
| POST /rla/v1/runs/{id}/complete | 3 accredited | JWT+SpiceDB | yes | Finalise run; compute conclusion |
| GET /rla/v1/runs/{id}/report | 1 signed-static | public | n/a | Published audit report (after completion) |
| GET /rla/v1/entropy/{source_id} | 1 signed-static | public | n/a | Public entropy seed record |
| GET /rla/v1/ballots/sample | 3 accredited | JWT+SpiceDB | n/a | Fetch next ballot sample during active run |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.rla.audit_run_started.v1 | emits | eel.rla.run.v1 | operational | yes | at-least-once idempotent |
| eel.rla.audit_run_completed.v1 | emits | eel.rla.run.v1 | operational | yes (HSM) | exactly-once |
| eel.rla.recount_triggered.v1 | emits | eel.rla.run.v1 | operational | yes (HSM) | exactly-once |
| eel.rts.form34b_accepted.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | tally totals + Form 34B pull |
| Public portal M11 | internal | integrations/public-portal/ | yes | published snapshot pull |
| Transparency log P05 | internal | integrations/tlog/ | yes | audit report anchoring |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on report publish |
| NIST randomness beacon | external | integrations/nist-beacon/ | yes (stub) | fallback to hardware dice |
| Election court M40 | internal | integrations/election-court/ | yes | recount-trigger handoff |

## ML and AI

none (all audit computations are statistical / deterministic; SHANGRLA or MINERVA algorithms)

## Dependencies

- Modules: M10 (tally data), M11 (published snapshot), M40 (election court interface for recount trigger)
- Platform: P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro

## Feature checklist

1. FR-M21-001: ballot-polling audit method (SHANGRLA) with configurable risk limit.
2. FR-M21-002: comparison audit method with reported vote totals from M10 Form 34B.
3. FR-M21-003: entropy source: NIST randomness beacon + hardware dice + witnessed ceremony; seed signed by >= 3 independent witnesses.
4. FR-M21-004: ballot sample generation: deterministic from seed; reproducible by any party.
5. FR-M21-005: auditor interpretation entry: paper ballot image shown; auditor records candidate marks.
6. FR-M21-006: discrepancy tracking: flag ballots where audit interpretation differs from reported tally.
7. FR-M21-007: audit conclusion: pass (risk limit met) or escalate (recount recommended).
8. FR-M21-008: recount trigger: if escalate, generate HSM-signed recount recommendation; forward to M40.
9. FR-M21-009: published audit report: HSM-signed PDF + JSON; anchored to tlog; publicly verifiable.
10. FR-M21-010: offline-capable Tauri desktop: auditor can work without network; sync on reconnect.
11. FR-M21-011: CLI tools: import/export audit data; verify seed; reproduce sample list.
12. FR-M21-012: multi-position: independent audit runs per elective position.
13. Entropy seed ceremony logged with witness signatures before sampling begins.
14. Audit run reproducible by any party with seed and tally snapshot.
15. Recount recommendation delivered to M40 within 4 h of audit completion.
16. Report published within 1 h of run completion.

## Acceptance criteria

AC-M21-01: audit sample reproducible from seed by independent party (reproducibility test).
AC-M21-02: recount trigger delivered to M40 when risk limit not met (integration test).
AC-M21-03: entropy ceremony requires >= 3 witness signatures (UAT).
AC-M21-04: offline desktop syncs audit data without loss on reconnect (chaos test).
AC-M21-05: published report anchored to tlog; chain verifiable (integration test).
AC-M21-06: SHANGRLA p-value computation matches NIST reference implementation (unit test).
AC-M21-07: 99.9% availability (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-01: offline Tauri; local queue; drain on reconnect.
EC-CON-06: NIST beacon unavailable; hardware dice fallback; ceremony documented.

**time**
EC-TIM-10: audit run must complete before petition window close; alert if behind schedule.

**identity**
EC-IDN-01: auditor cert revoked mid-run; halt; re-provision procedure.
EC-IDN-09: witness signing: >= 3 witnesses; no single-actor shortcut.

**data**
EC-DAT-01: duplicate ballot interpretation same key; cached.
EC-DAT-11: reported tally differs from Form 34B; discrepancy flagged; audit engine uses canonical 34B figure.

**security**
EC-SEC-01: report signature mismatch; CLI rejects.
EC-SEC-03: HSM unavailable; report publish blocked; alert.
EC-SEC-13: split-view on tlog; witness divergence.

**legal**
EC-LEG-09: recount recommendation legal basis: only IEBC Chair can act on it; recommendation is advisory.
EC-TIM-08: audit must complete within petition window T+7; scheduling constraint enforced.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | SHANGRLA p-value, sample generator, discrepancy counter, entropy combiner, report builder | nextest | M21-UNIT- |
| property | sample always reproducible from seed; p-value monotone with sample size | proptest | M21-PROP- |
| contract | M10 tally pull (consumer); M40 recount trigger (provider) | Pact | M21-PACT- |
| integration | M10 double; SoftHSM report sign; NIST beacon stub; tlog anchor | nextest + testcontainers | M21-INT- |
| e2e | start run -> entropy ceremony -> sample -> interpret -> conclude -> publish | Tauri test + Playwright | M21-E2E- |
| formal | SHANGRLA convergence property; entropy seed uniqueness | kani | M21-FORMAL- |
| security | seed tampering detection; report replay; ASVS L3 | Semgrep | M21-SEC- |
| load | 100k ballot interpretations per run; concurrent runs | k6 | M21-LOAD- |

## Runbooks and dashboards

- docs/runbooks/rla-audit-tool.md (create): entropy ceremony procedure; NIST beacon outage; recount trigger escalation; desktop offline sync.
- platform/observability/governance-svc/grafana-rla.json: audit run progress, discrepancy rate, p-value curve, entropy source status.
- platform/observability/governance-svc/slo-alerts.yaml: 99.9% burn rate; audit behind schedule alert; report publish > 1 h.

## Open questions

- Legal clarity on recount trigger: the Elections Act does not specify a binding threshold for IEBC to act on an RLA recommendation; IEBC legal team needs to clarify the legal effect of a recount recommendation before M reaches production.
- SHANGRLA vs MINERVA: both algorithms are implemented; the choice per election cycle must be decided by the audit committee before sampling begins; the tool supports both but defaults to SHANGRLA.
