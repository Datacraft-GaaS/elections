# M21 rla-audit-tool

| field | value |
|---|---|
| M-number | M21 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M21-rla-audit-tool.md |

## Purpose

Risk-Limiting Audit (RLA) tool: drive post-election statistical sampling of paper ballots against the digital tally using ballot-polling or comparison audit methods; generate NIST-compliant audit runs with cryptographically verifiable entropy sources; produce auditor reports and recount-trigger recommendations for the Commission and the courts.

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

Audit methods and setup:

FR-M21-001: Ingest published Form 34C JSON tally and its Merkle root as the starting point for the audit; validate Merkle inclusion proof before proceeding.
FR-M21-002: Ballot-polling audit method via SHANGRLA framework (Stark 2020); integrate upstream pbstark/audit-tools; all modifications published open-source under OSS licence.
FR-M21-003: Comparison audit method using reported vote totals from M10 Form 34B alongside SHANGRLA ballot-polling.
FR-M21-004: Sample size computation: margin of victory (from Form 34C), risk limit alpha (default 0.05, range 0.01-0.10), and contest structure as inputs; formula and all inputs displayed before drawing.
FR-M21-005: Multi-contest: independent audit runs per elective position (presidential, gubernatorial, senatorial, MP, women representative, MCA); independent alpha per contest.

Entropy and sample generation:

FR-M21-006: Five independent public entropy sources: NIST Randomness Beacon, UK National Lottery, BBC World Service shortwave hash, Kenya Gazette publication hash, SHA-256 of last Bitcoin block header at draw time.
FR-M21-007: All five source values with fetch timestamps recorded in audit log before seed computation; any fetch failure logged with rationale before fallback.
FR-M21-008: Composite seed: XOR of five sources (hex); any party with published values and software can reproduce exactly.
FR-M21-009: Deterministic ballot manifest: station code, ballot-box number, and sequential ballot position per sampled ballot; reproducible from seed alone on any POSIX system with Python 3.11+.
FR-M21-010: Manifest publication within 30 min of seed draw: on-screen sortable table, PDF download with QR code linking to manifest hash, HSM-signed JSON; all three published to public audit portal.
FR-M21-011: Entropy ceremony: seed draw requires IAB Commissioner HSM card and >=3 independent witness signatures before sampling begins; witnesses logged with identity and time.
FR-M21-012: Air-gapped environment for seed draw and sample computation; one-way data diode for public portal publication.
FR-M21-013: Sample manifest computation <=60 s for 55,393 stations on reference hardware (2 vCPU, 8 GB RAM).

Ballot retrieval and tally:

FR-M21-014: Ballot-retrieval checklist per sampling-team member: station name, GPS coordinates link, ballot-box number, expected seal numbers; QR-code scan-to-confirm for each retrieved ballot.
FR-M21-015: 48-hour advance notice delivered to presiding officers at sampled stations; ballot boxes must remain sealed and accessible.
FR-M21-016: Tally interface: auditor records candidate marks per retrieved ballot (or undervote, overvote, unreadable); Form 34A tallies shown read-only alongside for comparison.
FR-M21-017: Party-agent witness counter-signature on each retrieved ballot retrieval record before tally is locked.
FR-M21-018: Tally entry immutable after party-agent counter-signature; any modification attempt logged and rejected.
FR-M21-019: Discrepancy tracking: flag every ballot where audit interpretation differs from reported tally; discrepancy count displayed in real time.
FR-M21-020: Running SHANGRLA T-statistic computed after each batch of tallied ballots; current confidence level, remaining sample needed, and stopping condition status displayed.

Conclusions and reporting:

FR-M21-021: Stopping condition: if T-statistic fails to confirm at risk limit after maximum sample exhausted, RECOUNT_REQUIRED event generated immediately; published to the Commission Chair, IAB, and public portal; HSM-signed; forwarded to M40 election court interface.
FR-M21-022: Audit conclusion: CONFIRMED (risk limit met) or RECOUNT_REQUIRED; conclusion computed deterministically from published inputs.
FR-M21-023: Signed audit report on completion: seed values, sample manifest hash, tally data, T-statistic trajectory, outcome, party-agent and IAB signatures; HSM-signed PDF + JSON.
FR-M21-024: Audit report published on public portal and submitted to Supreme Court IT Registrar by T+7 days post-election.
FR-M21-025: Cryptographic sortition log appended to P05 Merkle chain within 1 h of seed draw.

Reproducibility and offline:

FR-M21-026: Reproduce-me script published alongside every audit; bit-identical output on any POSIX system; CI test verifies on every release.
FR-M21-027: Offline-capable Tauri desktop: complete statistical computation and tally entry without network; signed results sync on reconnect.
FR-M21-028: CLI tools: import/export audit data; verify seed; reproduce sample manifest; verify report signature.

Accessibility and DPA:

FR-M21-029: WCAG 2.2 AA for all auditor-facing and public-facing screens; EN + SW for auditor interface; EN + SW + per-county Kenyan language for public portal.
FR-M21-030: DPA 2019: audit log contains no voter-identifiable data; sampled-ballot records contain station code and ballot position only.
FR-M21-031: Audit log immutability: append-only; every entry SHA-256-chained; deletion or modification detectable; chain-break triggers P0 alert.

## Acceptance criteria

AC-M21-01: Ballot sample reproducible from seed; independent party running reproduce-me script gets byte-identical manifest; CI gate enforces on every release (reproducibility test).
AC-M21-02: RECOUNT_REQUIRED event delivered to M40 and public portal with HSM signature when risk limit not met after maximum sample exhausted (integration test).
AC-M21-03: Entropy ceremony requires IAB Commissioner HSM card and >=3 witness signatures before sampling proceeds; single-officer attempt rejected (UAT + security test).
AC-M21-04: Tally entry immutable after party-agent counter-signature; POST to modify returns 403 and audit-log entry (security test).
AC-M21-05: Offline Tauri desktop completes tally entry and syncs signed results without loss after 48-h network partition (chaos test).
AC-M21-06: Published audit report anchored to P05 Merkle chain; inclusion proof verifiable by any party using published chain (integration test).
AC-M21-07: SHANGRLA T-statistic matches NIST reference implementation for all published test vectors to at least 15 significant figures (unit test).
AC-M21-08: Sample manifest published within 30 min of seed draw; timestamp delta verified by integration test against 30-min deadline.
AC-M21-09: Audit report delivered to Supreme Court IT Registrar signed receipt obtained by T+7 days post-election; process audit.
AC-M21-10: Sample manifest computation <=60 s for 55,393 stations on reference hardware (2 vCPU, 8 GB RAM); performance test on every release.
AC-M21-11: Public audit portal available >=99.9% during 30-day petition window (cycle-critical SLO per appendix; burn-rate alert fires before budget exhausted).
AC-M21-12: WCAG 2.2 AA: axe-core passes on all auditor-facing and public-facing screens; manual screen-reader verification with NVDA and ChromeVox (accessibility audit).

## Applicable edge cases

EC-CON-01: Tauri desktop offline for 48+ h; tally entry proceeds in local queue; signed results sync on reconnect; no data loss; ECC receives historical snapshots on reconnection.
EC-CON-06: NIST Randomness Beacon unreachable at seed draw; 15-min retry window; fallback to cached NIST value from prior 24 h with published rationale; other four entropy sources still provide full diversity.
EC-CON-08: DNS poisoned during entropy source fetch; DNSSEC enforced; hardcoded fallback NIST beacon endpoint; fetch failure logged with rationale.
EC-TIM-06: Audit event time < causation time (delayed sync from offline Tauri); accepted; audit skew anomaly logged; T-statistic projection independent of wall order.
EC-TIM-08: Audit must complete and report delivered to Supreme Court IT Registrar by T+7 (statutory petition window); scheduling constraint enforced; alert fires at T+5 if completion risk detected.
EC-DAT-01: Duplicate ballot tally interpretation same idempotency key and body; cached response returned; T-statistic not double-counted.
EC-DAT-02: Same ballot tally key, different interpretation body; rejected 422 idempotency-key-reuse before counter-signature.
EC-DAT-03: Out-of-order tally entries in stream (offline sync); buffered until sequence gap filled or DLQ after timeout.
EC-DAT-04: Replayed signed entropy event; rejected on sequence counter; tamper topic populated.
EC-DAT-05: Poison tally value (e.g., candidate ID not in contest); DLQ with reason; audit run continues with remaining ballots.
EC-DAT-06: Schema v1/v2 coexist during rolling deploy; both accepted; dual-emit; Karapace compat check.
EC-DAT-09: Unicode candidate names in tally interface (all 11 locales); NFC normalisation; no truncation on tally-entry screen or report.
EC-DAT-10: Zero discrepancies is a valid and expected outcome; renders "0 discrepancies detected" with confidence level; no divide-by-zero in T-statistic path.
EC-DAT-11: Reported tally in Form 34C differs from Form 34B; discrepancy flagged; audit engine uses canonical Form 34C as audit input; discrepancy logged for investigator review.
EC-DAT-13: Soft-deleted (voided) audit run appears in search; tombstone honoured; excluded from active conclusions.
EC-IDN-02: JWT expires mid-tally entry; in-flight request completes; next request returns 401 with silent-refresh hint.
EC-IDN-09: Party-agent witness revoked between ballot retrieval and counter-signature; re-validate at commit; IAB observer replaces per MoU affidavit procedure.
EC-IDN-10: SpiceDB down; fail closed on tally writes and report publish; class-C reads (public manifest, entropy source list) allowed.
EC-SEC-01: Report signature mismatch on CLI reproduce-me verification; tampering immediately evident; verification fails with clear error; alert.
EC-SEC-03: HSM unavailable; report publish blocked; seed draw blocked; ingest continues; alert to IAB; degraded mode documented in runbook.
EC-SEC-05: Injection on tally entry free-text field (unreadable reason); parameterised queries only; Semgrep compile-time and ZAP dynamic scan required.
EC-SEC-13: Split-view on P05 tlog; witness divergence detected between IAB domain and GitHub channel; portal shows discrepancy; IAB notified.
EC-SEC-16: Secrets in audit logs or traces refused; telemetry redacted; log-scanner fixture in CI.
EC-SCL-04: Consumer lag >60 s on eel.rla.run.v1 topic; backpressure activated; page fired; no OOM.
EC-LEG-04: Recount ordered post-audit; compensating-event version created; original and recount conclusions both shown with Merkle proofs; append-only invariant verified.
EC-HUM-06: Screen readers: audit portal fully navigable; WCAG 2.2 AA verified with axe-core and manual NVDA test; tally entry form operable via keyboard only.

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

- Legal clarity on recount trigger: the Elections Act does not specify a binding threshold for the Commission to act on an RLA recommendation; the Commission legal team needs to clarify the legal effect of a recount recommendation before M reaches production.
- SHANGRLA vs MINERVA: both algorithms are implemented; the choice per election cycle must be decided by the audit committee before sampling begins; the tool supports both but defaults to SHANGRLA.
