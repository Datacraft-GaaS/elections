# Transmission legal basis audit (M62)

Module M62 (`transmission-legal-basis-audit`) owned by `governance-svc`. Verifies 100% of RTS (Results Transmission System) transmission events against a statutory legal rules engine, records a `ComplianceVerdict` for each event, and monitors for rule drift (rules that were compliant when the transmission occurred but are now non-compliant due to a subsequent regulatory change).

## Contents

1. What Transmission Legal Basis Audit does (vs M10 RTS backend)
2. Data plane (schema)
3. Rules engine
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Transmission Legal Basis Audit does (vs M10 RTS backend)

- **M10** (RTS backend) = technical transmission: receives signed Form 34A batches, verifies signatures, aggregates results. It enforces technical correctness.
- **M62** = legal correctness: for every transmission event emitted by M10, it evaluates whether the transmission complied with every applicable legal rule (timing windows, authorisation chains, format requirements, geographic constraints) and records a verdict.

M62 is a subscriber to M10's event stream; it does not block or modify M10's processing. A non-compliant verdict is a flag for legal review, not a technical rollback.

## 2. Data plane

Three aggregates (per `services/governance-svc/crates/transmission-audit/src/models.rs` and the M62 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **LegalRule** | A single statutory or regulatory rule: `rule_id`, `rule_code` (e.g., `EA-S74-TIMING`), `description`, `source` (Elections Act section / regulation number), `predicate_expr` (CEL expression, evaluated against TransmissionEvent context), `effective_from`, `effective_to`, `version`. | A |
| **TransmissionEvent** | Mirror of the M10 event with additional context: `event_id` (same as M10 CloudEvent ID), `station_code`, `form_type` (F34A / F34B / F35), `transmitted_at`, `presiding_officer_id`, `signature_valid` (from M10), `received_at_backend`. | A |
| **ComplianceVerdict** | Result of evaluating a TransmissionEvent against all active LegalRules: `verdict_id`, `event_id`, `evaluated_at`, `rule_verdicts` (JSONB: array of `{rule_code, compliant, explanation}`), `overall_compliant`, `flagged_rules` (array of non-compliant rule codes). | A |

## 3. Rules engine

Rules are expressed in Google Common Expression Language (CEL; Kuznetsov et al., Google, 2021). Each `LegalRule.predicate_expr` is a CEL expression evaluated against a context object containing the `TransmissionEvent` fields plus election calendar data (poll open time, poll close time, form submission deadline).

Example rules:

| Rule code | Predicate (CEL) | Source |
|---|---|---|
| `EA-S74-TIMING` | `event.transmitted_at >= election.polls_open && event.transmitted_at <= election.form_deadline` | Elections Act §74 |
| `EA-S74-SIGNATURE` | `event.signature_valid == true` | Elections (Technology) Regulations 2017 Reg 5 |
| `EA-S39-FORM-TYPE` | `event.form_type in ['F34A', 'F34B', 'F35']` | Elections Act §39 |

Rules are versioned; a `LegalRule` with `effective_to` in the past is inactive but retained for historical verdict replay.

Rule drift monitor: a nightly job re-evaluates all `TransmissionEvent` records from the current election cycle against the current active rule set. If a previously compliant event is now non-compliant under a new or amended rule, a `rule_drift_alert` is emitted and the `ComplianceVerdict` is re-evaluated with a `drift_detected` flag.

## 4. Pipeline

Files on disk under `services/governance-svc/crates/transmission-audit/src`: `lib.rs`, `models.rs`, `rules_engine.rs`, `verdicts.rs`, `drift_monitor.rs`, `routes/`.

```
M10 emits eel.rts.transmission_event.v1 (for every Form 34A/34B/35)
      -> M62 consumer receives event via Kafka
      -> create TransmissionEvent mirror
      -> load all active LegalRules (CEL cache warm)
      -> evaluate each rule against event context:
            for each rule:
              compliant = CEL.evaluate(rule.predicate_expr, context)
              rule_verdicts[rule.rule_code] = {compliant, explanation}
      -> overall_compliant = all(rule_verdicts[*].compliant)
      -> create ComplianceVerdict
      -> if overall_compliant == false:
            -> emit eel.governance.transmission_non_compliant.v1
            -> legal officer queue updated
      -> emit eel.governance.transmission_verdict_recorded.v1

nightly rule drift monitor:
      -> for each TransmissionEvent in current cycle:
            -> re-evaluate against current active rules
            -> if verdict changes: emit eel.governance.rule_drift_detected.v1
            -> update ComplianceVerdict with drift_detected = true
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/transmission-audit.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/transmission-audit/verdicts` | Legal officer (JWT + SpiceDB `LEGAL_OFFICER`) | List verdicts; filter by overall_compliant |
| `GET /v1/transmission-audit/verdicts/{id}` | Legal officer, OAG auditor | Full verdict detail + per-rule breakdown |
| `GET /v1/transmission-audit/verdicts/{id}/event` | Legal officer | Underlying TransmissionEvent |
| `POST /v1/transmission-audit/rules` | Legal config officer (SpiceDB `LEGAL_CONFIG_OFFICER`) | Add new LegalRule |
| `PATCH /v1/transmission-audit/rules/{id}/expire` | Legal config officer | Expire a rule (set effective_to) |
| `GET /v1/transmission-audit/rules` | Legal officer | List all rules (active + historical) |
| `GET /v1/transmission-audit/drift-alerts` | Legal officer | Rule drift alerts for current cycle |
| `POST /v1/transmission-audit/verdicts/replay` | Legal config officer | Trigger full verdict replay (after rule change) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Legal compliance console (Django) | `services/governance-svc/django/transmission_audit_admin` | Manage rules, review non-compliant verdicts, act on drift alerts |
| Rule authoring workspace (LiveView) | `services/governance-svc/ex/rule_web` | CEL rule authoring with live test against a sample event |

Languages: EN. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Immutable verdicts**: `ComplianceVerdict` rows are insert-only (no UPDATE path exists in the application). Drift re-evaluation creates a new `ComplianceVerdict` version linked via `parent_verdict_id`, not an in-place edit.
- **CEL sandbox**: the CEL evaluator runs in a sandboxed environment with no file system or network access; rules cannot perform side effects.
- **Rule change audit**: every `LegalRule` modification (creation, expiry) is logged to `eel_governance_audit_shadow` with the legal config officer's identity and timestamp.
- **Verdict integrity hash**: `ComplianceVerdict.verdict_hash` = SHA-256 over `(event_id || evaluated_at || rule_verdicts JSONB || overall_compliant)`. A nightly job re-hashes stored verdicts and alerts on mismatch.
- **100% coverage assertion**: a Prometheus metric `eel_transmission_audit_coverage_ratio` measures `verdicts / transmission_events` for the current election cycle. An alert fires if the ratio drops below 1.0 for more than 5 minutes.

## 8. Kenya statute alignment

- **Elections Act §74 (transmission requirements)**: the timing, signature, and form-type rules in the rules engine directly implement §74 requirements.
- **Elections (Technology) Regulations 2017 Regulation 5 (digital signatures)**: `EA-S74-SIGNATURE` rule enforces this requirement for every transmission event.
- **Evidence Act Cap 80 §78 (electronic records)**: `ComplianceVerdict` with its `verdict_hash` provides an audit-grade record of compliance at the time of transmission, admissible in election petition proceedings.
- **Constitution of Kenya 2010 Art 86(c) (accurate tallying)**: the verdicts provide the legal audit trail that every tally was transmitted in compliance with statutory requirements.

## 9. Honest caveats + external prerequisites

- CEL rule authoring requires legal staff with sufficient technical literacy to write predicate expressions; the LiveView rule authoring workspace provides test-against-sample functionality to reduce authoring errors.
- Rule drift is flagged but not automatically resolved; legal officers must review drifted verdicts and determine whether remedial action (petition to court, regulatory clarification) is required.
- The system evaluates legal rules as configured by the Commission legal officers; it is not a substitute for legal advice. Rules must be reviewed by the Commission legal team and external counsel before each election cycle.
- Coverage SLA (100%) assumes Kafka consumer lag is below 60 seconds; sustained Kafka lag can cause the coverage ratio to drop below 1.0 and trigger false positive alerts.
- Historical verdict replay (after a rule change) is a long-running job; for a full election cycle (~50k transmission events), replay takes approximately 30 minutes.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/transmission-audit/tests/`: CEL evaluation correctness (property-based, fuzz rules against known-good and known-bad events), immutable verdict enforcement (integration: attempt UPDATE must fail), 100% coverage assertion (integration: emit 100 events, assert 100 verdicts), rule drift detection (integration: change a rule, run nightly job, assert drift alert), verdict hash integrity (property-based).

Runbooks under `docs/runbooks/`:

- `governance-svc.md` (section: transmission-legal-basis-audit) -- deploy, rollback, rule change procedure.
- `coverage-ratio-breach.md` -- steps when audit coverage drops below 100%.
- `rule-drift-response.md` -- legal officer response procedure for drift alerts.

Related documents: full spec at `docs/plans/modules/transmission-legal-basis-audit.md`; service capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M10 RTS backend at `docs/features/rts-backend.md`.

### Key metrics and SLOs

| Metric | Target | Alert threshold |
|---|---|---|
| Verdict coverage ratio | 1.0 (100% of transmission events) | < 1.0 for > 5 min: PagerDuty |
| Verdict latency (event to verdict) | <= 60 s | > 120 s: alert |
| Non-compliant verdict rate | 0% for certified election | Any non-compliant: legal officer queue |
| Rule drift alerts per cycle | 0 (target) | Any: legal officer review within 24 h |

### CEL evaluation performance

The CEL evaluator (`google/cel-go` bindings via the `cel-rust` crate) compiles each `LegalRule.predicate_expr` to a native AST at rule creation time; evaluation is bytecode-interpreted rather than re-parsed on every event. With 20 active rules, evaluation time is approximately 2 ms per event on a single core. At peak M10 transmission volume (~5,000 Form 34A events in 2 hours at close of poll), the M62 consumer group of 4 workers handles the load with < 10 s total processing time. The 60-second verdict latency SLA is therefore conservative.

CEL rule expressions are validated at creation via a dry-run against a fixed sample event; invalid CEL expressions are rejected with HTTP 422 and the specific syntax error before the rule is saved.
