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

Form receipt and ICR review:
1. FR-M09-001: receive signed Form 34A batches from M10; verify kit signature before display. (spec FR-M09-001)
2. FR-M09-002: ICR-assisted count extraction with highlighted low-confidence fields; prompt RO to confirm or correct with reason. (spec FR-M09-002 / spec FR-M09-003)
3. FR-M09-003: station-by-station comparison: extracted counts vs kit-reported counts; flag discrepancies for RO review.
4. FR-M09-004: human review mandatory for low-confidence ICR extractions (below configurable threshold) and for random sample of high-confidence forms.
5. FR-M09-005: ICR confidence threshold configurable without code deploy; hot-reload via config bundle.
6. FR-M09-006: ICR correction captured as adjudicator decision with confidence delta; streamed back to M10 as training signal.
7. FR-M09-007: side-by-side display of ICR extraction and original form scan image; RO confirms or edits.

Disputes:
8. FR-M09-008: dispute workflow: raise dispute on Form 34A or 34B; upload evidence; inter-party witness; resolution tracked. (spec FR-M09-004)
9. FR-M09-009: dispute escalation: unresolved disputes after configurable timeout auto-escalate to national tally board.
10. FR-M09-010: party agent may flag a Form 34A or 34B for dispute; workflow escalates without blocking other stations.

Aggregation and signing:
11. FR-M09-011: Form 34B assembly: aggregate station subtotals; totals locked when all stations adjudicated. (spec FR-M09-005)
12. FR-M09-012: Form 34C assembly (regional): aggregate 34B totals per region; regional RO signs. (spec FR-M09-006)
13. FR-M09-013: multi-position support: simultaneous independent 34B/34C pipelines per elective position.
14. FR-M09-014: validation rules enforced: votes cast <= registered voters; rejected + valid = total; non-negative counts. (spec, validation)
15. FR-M09-015: dual-officer signing for 34B/34C: primary RO plus witness with separate HSM keys; physically separate auth challenges. (spec FR-M09-007)
16. FR-M09-016: zero silent auto-tally: every constituency total requires RO signature or explicit documented skip with dual-officer justification.
17. FR-M09-017: emergency close of constituency count with dual-control justification recorded. (spec FR-M09-015)

Publication:
18. FR-M09-018: publish signed 34B/34C to M10 tally and M11 public portal with Merkle-inclusion proof. (spec FR-M09-008)
19. FR-M09-019: party agent countersignature on 34B/34C: optional, within configurable window post-RO-sign. (spec FR-M09-013)
20. FR-M09-020: print certified 34B/34C with QR watermark for physical distribution to agents. (spec FR-M09-014)
21. FR-M09-021: PDF and CSV export per gazette template for archival. (spec FR-M09-012)

Session recording and audit:
22. FR-M09-022: session recording: all screen actions encrypted with dual custody; anchor to Merkle root every 60 s. (spec FR-M09-014)
23. FR-M09-023: real-time tallying progress dashboard: expected vs reported station count per elective position. (spec FR-M09-002)
24. FR-M09-024: offline-capable Tauri desktop: local event queue survives network loss; drain on reconnect without loss. (spec FR-M09-010)

Accessibility and language:
25. FR-M09-025: WCAG 2.1 AA on Tauri desktop UI; screen-reader support on all officer-facing surfaces.
26. FR-M09-026: 11-language support on all officer-facing screens; no hardcoded strings.

Compliance:
27. FR-M09-027: ODPC data flows: RO identity in signed events covered by DPIA; data minimisation applied.
28. FR-M09-028: audit export: RO can export their session events for self-review and court submission.
29. FR-M09-029: participation token count shown alongside station tally; cross-referenced against M14 bundle.
30. FR-M09-030: Elections Act s.39 compliance: Form 34B/34C published with statutory deadline tracking.

## Acceptance criteria

AC-M09-01: dual-officer signing requires separate HSM challenges; single officer cannot complete (UAT).
AC-M09-02: ICR correction captured in audit log with confidence delta (integration test).
AC-M09-03: dispute raised, evidenced, resolved, and reflected in Form 34B (UAT).
AC-M09-04: offline queue drains without data loss after 7-day accumulation (chaos exercise).
AC-M09-05: Form 34C aggregates 34B totals accurately across all positions (property test).
AC-M09-06: published 34B/34C appears on public portal (M11) within 60 s (integration test).
AC-M09-07: session audit log anchored to transparency log; chain verifiable (integration test).
AC-M09-08: 99.99% availability over tallying window (SLO measurement).
AC-M09-09: Form 34B generated within 60 s of last station adjudicated (UAT).
AC-M09-10: 290 constituency RO simulations in parallel in pre-production; p99 submission <= 2 s (load test).
AC-M09-11: WCAG 2.1 AA: axe zero violations on Tauri desktop; screen-reader traversal of tally flow (accessibility audit).
AC-M09-12: 11 locales: no truncated strings on printed 34B/34C; ICU plural forms correct (i18n test).
AC-M09-13: zero silent auto-tally: every constituency total has RO signature or documented skip in audit log (integration test).

## Applicable edge cases

connectivity:
EC-CON-01: offline Tauri; local event queue; drain on reconnect without gap.
EC-CON-09: tunnel drop mid-publish; resume at sequence; idempotent re-send.

time:
EC-TIM-01: desktop clock skew; HLC from M10 reference; skew alert on large delta.
EC-TIM-02: NTP/NTS unreachable all day; last-auth time plus monotonic; time_confidence=low.
EC-TIM-03: back-end clock jumps backward; HLC never regresses; skew alert.
EC-TIM-04: DST/TZ: tallying deadline computed in UTC; displayed in EAT.
EC-TIM-05: leap second in event timestamp; tolerates 23:59:60 in parser.
EC-TIM-06: event time < causation time in event log; accept; audit skew anomaly.
EC-TIM-07: extended tallying window; signed extension event consumed; fence reopened.
EC-TIM-08: tallying deadline s.75(1); signed deadline extension or close with partial.
EC-TIM-09: register freeze race at T-60; OPA hard-close; all-or-nothing aggregation.

identity:
EC-IDN-01: RO cert revoked mid-session; halt; escalate to M16 ECC.
EC-IDN-02: JWT expires mid-request; in-flight completes; next request 401 + silent refresh.
EC-IDN-03: SVID rotates; old valid to expiry; producer_key_id per event.
EC-IDN-09: dual-officer signing: re-validate both at commit; approver revocation mid-flow causes re-delegation.
EC-IDN-10: SpiceDB down; fail closed on 34B publish writes; class-C reads continue.

data:
EC-DAT-01: duplicate 34B publish same key+body; cached response; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-03: out-of-order Form 34A in stream; buffer until gap filled or DLQ.
EC-DAT-04: replayed signed event; reject on monotonic seq counter.
EC-DAT-05: poison Form 34A (votes > registered); DLQ with reason; partition continues.
EC-DAT-06: schema v1/v2 coexist in stream; both accepted; dual-emit; compat check.
EC-DAT-07: partial batch failure during aggregation; per-row report; valid rows committed.
EC-DAT-09: Unicode candidate names on 34B; NFC normalisation; grapheme-safe.
EC-DAT-10: zero votes for candidate; valid; renders "0"; no div-by-zero.
EC-DAT-11: extracted counts > registered voters; block; RO override creates DisputeCase.
EC-DAT-12: Debezium slot lag; alert 60 s; outbox retained; disk guard.
EC-DAT-14: oversized ICR evidence payload > 1 MB; blob to MinIO + hash reference.

security:
EC-SEC-01: signature mismatch on form receipt; never applied; tamper alert.
EC-SEC-02: tamper of Tauri binary; integrity check on launch; refuse if digest mismatch.
EC-SEC-03: HSM unavailable; 34B/34C publishing blocked; incident escalated to ECC.
EC-SEC-05: injection on ICR correction inputs and dispute reason fields; parameterised only.
EC-SEC-16: secrets in logs refused; RO identity hash not plaintext in telemetry.

legal:
EC-LEG-01: candidate withdrew post-print; status frozen; tallied + annotated in 34B.
EC-LEG-03: s.44A invoked; dual S44AEvent; manual count logged separately.
EC-LEG-04: recount ordered; new version via compensating event; both versions shown with proofs.

operations:
EC-OPS-01: power loss mid-34B assembly; journal-before-act; exact resume.
EC-OPS-02: desktop swap; event log transferred; old session revoked.
EC-OPS-04: dual-control, approver absent; time-boxed ECC remote co-sign.
EC-OPS-05: power outage at tally centre; SQLite WAL resume; no duplicate 34B.

scale:
EC-SCL-04: consumer lag > 60 s on form topic; backpressure applied; no OOM on aggregator.
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
