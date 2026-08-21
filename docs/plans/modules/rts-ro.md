# M09 rts-ro

| field | value |
|---|---|
| M-number | M09 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.2 (docs/plans/04-wave-3-critical-path.md) |
| Size | M |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M09-rts-ro.md |

## Purpose

The Returning Officer tallying station receives signed Form 34A batches from kits (via M10), runs ICR-assisted extraction, adjudicates disputes, assembles Form 34B constituency results and Form 34C regional results, and publishes them under HSM-backed dual-officer signatures before forwarding to the national tally.

## Surfaces and languages

| surface | technology | path under services/rts-svc/ |
|---|---|---|
| RO desktop application | Tauri (Rust shell + Astro/LiveView frontend) | apps/rts-ro-desktop/ |
| RO API gateway | Rust (axum + utoipa) | crates/rts-ro-api/ |
| Domain + adjudication logic | Rust | crates/rts-domain/ |
| ICR correction UI | Astro embedded in Tauri | apps/rts-ro-desktop/src/icr/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Form34A | form_id, station_code, election_id, position_code, raw_scan_ref, extracted_counts, status, dispute_case_id | C | T1-audit |
| Form34B | form_id, constituency_code, election_id, position_code, station_subtotals, totals, ro_sig, witness_sig | C | T1-audit |
| Form34C | form_id, region_code, election_id, constituency_subtotals, totals, ro_sig, witness_sig | C | T1-audit |
| ICRExtraction | extraction_id, form_id, raw_text, corrected_text, confidence, corrected_by, corrected_at | C | T1-audit |
| DisputeCase | case_id, form_id, raised_by, reason, evidence_refs, resolution, closed_at | C | T1-audit |
| SessionAuditEvent | event_id, actor, action, ts_hlc, session_sig, seq | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/rts/ro.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /rts/v1/forms/34a | 3 accredited | JWT+SpiceDB | n/a | Retrieve batch of signed 34As for this constituency |
| POST /rts/v1/icr/{form_id}/corrections | 3 accredited | JWT+SpiceDB | yes | Submit ICR-corrected counts |
| POST /rts/v1/disputes | 3 accredited | JWT+SpiceDB | yes | Open dispute case on a form |
| PATCH /rts/v1/disputes/{id} | 3 accredited | JWT+SpiceDB | yes | Resolve or escalate dispute |
| POST /rts/v1/forms/34b | 3 accredited | JWT+SpiceDB | yes | Publish signed Form 34B (dual-officer) |
| POST /rts/v1/forms/34c | 3 accredited | JWT+SpiceDB | yes | Publish signed Form 34C (dual-officer) |
| GET /rts/v1/tally/progress | 3 accredited | JWT+SpiceDB | n/a | Real-time tallying progress for constituency |
| POST /rts/v1/session/record | internal gRPC | SVID mTLS | yes | Session audit event (screen-capture anchor) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.rts.form34a_received.v1 | emits | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.icr_correction_applied.v1 | emits | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.dispute_opened.v1 | emits | eel.rts.dispute.v1 | event-source | yes | exactly-once |
| eel.rts.dispute_resolved.v1 | emits | eel.rts.dispute.v1 | event-source | yes | exactly-once |
| eel.rts.form34b_published.v1 | emits | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.form34c_published.v1 | emits | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.form_captured.v1 | consumes | eel.rts.form.v1 | operational | yes | exactly-once |
| eel.rts.transmission_succeeded.v1 | consumes | eel.rts.form.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end (M10) | internal | integrations/rts-backend/ | yes | offline queue in Tauri; drain on reconnect |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed: 34B/34C cannot publish without HSM |
| Transparency log (P05) | internal | integrations/tlog/ | yes | anchoring queued; retry |
| ICR engine (M10) | internal | integrations/icr/ | yes (stub) | graceful degradation to manual entry |

## ML and AI

ICR (intelligent character recognition) for handwritten tallies is invoked server-side in M10; this module shows corrections in the UI and captures adjudicator decisions as training signals (streamed back to M10).

## Dependencies

- Modules: M08 (form source), M10 (ICR + ingest + back-end event source), M21 (RLA audit uses 34B/34C)
- Platform: P05 (audit/Merkle), P06 (HSM dual-officer signature)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs

## Feature checklist

1. FR-M09-001: receive signed Form 34A batches from M10; verify kit signature before display.
2. FR-M09-002: ICR-assisted count extraction; highlight low-confidence fields; prompt RO correction.
3. FR-M09-003: station-by-station comparison: extracted vs kit-reported; flag discrepancies.
4. FR-M09-004: dispute workflow: raise, evidence upload, inter-party witness, resolution.
5. FR-M09-005: Form 34B assembly: aggregate station subtotals; totals locked when all stations adjudicated.
6. FR-M09-006: Form 34C assembly (regional): aggregate 34B totals; regional RO signs.
7. FR-M09-007: dual-officer signing for 34B/34C: primary RO + witness with separate HSM keys.
8. FR-M09-008: published 34B/34C forwarded to M10 tally and M11 public portal.
9. FR-M09-009: session recording: all user actions logged and anchored to Merkle root for audit.
10. FR-M09-010: offline-capable Tauri desktop; local event queue survives network loss.
11. FR-M09-011: real-time tallying progress dashboard for constituency.
12. FR-M09-012: multi-position support: simultaneous 34B/34C per position.
13. FR-M09-013: party agent countersignature on 34B/34C (optional, within window).
14. FR-M09-014: print certified 34B/34C with QR watermark for physical distribution.
15. Dispute UX: every step recorded with actor + timestamp + ICR confidence score.
16. Dual-officer signing workflow requires physically separate auth challenges.
17. p95 form retrieval <= 2 s; p95 34B publish <= 5 s.
18. Session recording screen-capture anchored every 60 s.
19. Offline queue holds up to 7 days; drains without data loss on reconnect.

## Acceptance criteria

AC-M09-01: dual-officer signing requires separate HSM challenges; single officer cannot complete (UAT).
AC-M09-02: ICR correction captured in audit log with confidence delta (integration test).
AC-M09-03: dispute raised, evidenced, resolved, and reflected in Form 34B (UAT).
AC-M09-04: offline queue drains without data loss after 7-day accumulation (chaos exercise).
AC-M09-05: Form 34C aggregates 34B totals accurately across all positions (property test).
AC-M09-06: published 34B/34C appears on public portal (M11) within 60 s (integration test).
AC-M09-07: session audit log anchored to transparency log; chain verifiable (integration test).
AC-M09-08: 99.99% availability over tallying window (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-01: offline Tauri; local event queue; drain on reconnect.
EC-CON-09: tunnel drop mid-publish; resume at sequence; idempotent.

**time**
EC-TIM-01: desktop clock skew; HLC from M10 reference; skew alert.
EC-TIM-07: extended tallying window; signed extension event consumed.

**identity**
EC-IDN-01: RO cert revoked mid-session; halt; escalate to M16 ECC.
EC-IDN-03: SVID rotates; old valid to expiry; producer_key_id per event.
EC-IDN-09: dual-officer: re-validate both at commit; no surrogate signing.

**data**
EC-DAT-01: duplicate 34B publish same key+body; cached.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-04: replayed signed event; reject on seq counter.
EC-DAT-11: extracted counts > registered voters; block; RO override -> DisputeCase.

**security**
EC-SEC-01: signature mismatch on form receipt; never applied; tamper alert.
EC-SEC-02: tamper of Tauri binary; integrity check on launch.
EC-SEC-03: HSM unavailable; publishing blocked; incident escalated to ECC.
EC-SEC-16: secrets in logs refused.

**legal**
EC-LEG-01: candidate withdrew post-print; status frozen; tallied + annotated in 34B.
EC-LEG-03: s.44A invoked; dual S44AEvent; manual count logged separately.
EC-TIM-08: tallying deadline (s.75(1)); signed deadline extension or close with partial.

**operations**
EC-OPS-01: power loss mid-34B assembly; journal-before-act; exact resume.
EC-OPS-02: desktop swap; event log transferred; old session revoked.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ICR correction logic, dispute FSM, dual-officer signing flow, 34B/34C aggregation | nextest | M09-UNIT- |
| property | 34B totals = sum of 34A station counts; dispute always resolves or escalates; HLC monotonic | proptest | M09-PROP- |
| contract | M10 form ingest (consumer); M11 publication event (provider) | Pact | M09-PACT- |
| integration | ICR engine stub; HSM SoftHSM; offline drain; dispute workflow | nextest + testcontainers | M09-INT- |
| e2e | full RO session: receive -> ICR -> dispute -> 34B dual-sign -> portal publish | patrol + Playwright | M09-E2E- |
| fuzz | ICR correction payload; dispute evidence; form counts | cargo-fuzz | M09-FUZZ- |
| formal | dual-officer invariant; 34B = sum(34A); no publish without resolution | kani | M09-FORMAL- |
| load | 50 constituencies x 200 stations tallying concurrently | k6 | M09-LOAD- |
| chaos | power cut; HSM kill; network drop mid-sign; desktop swap | netem + kill -9 | M09-CHAOS- |
| security | ASVS L3; STRIDE on dual-sign flow; Semgrep; binary integrity | Semgrep | M09-SEC- |

## Runbooks and dashboards

- docs/runbooks/rts-svc-ro-degradation.md (create): HSM fail procedure; dual-officer substitute; session recording gap handling.
- platform/observability/rts-svc/grafana-ro-tally.json: form receipt rate, ICR confidence distribution, dispute queue depth, 34B/34C publish latency.
- platform/observability/rts-svc/slo-alerts.yaml: 99.99% burn rate; form retrieval p95 > 2 s; 34B publish p95 > 5 s.

## Open questions

- Session recording: screen-capture anchor cadence (60 s) needs IEBC legal sign-off on retention period and access controls.
- ICR confidence threshold: the threshold below which manual correction is mandatory needs tuning on real Kenyan handwriting samples before G3.
