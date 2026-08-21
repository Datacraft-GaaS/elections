# eel-governance-legal-audit

M62 Transmission legal basis audit: Elections Act §39(1C) compliance verification, real-time checks, evidence citation.

## What it does

Validates transmission events (Form 34A, Form 34B, Form 34C) against Elections Act §39(1C) legal requirements at submission time. Rules: transmission must originate from a registered polling station, use authorized signatory, occur within declared election window, and include mandatory supporting evidence (signed Form 35). On violation, issues ComplianceVerdict with evidence citations and escalates to governance-review for manual investigation.

Fits in governance-svc as the real-time compliance engine. Runs on port 8093 (shared service); consumes rts.form34.published events; emits governance.transmission.verdict.issued.

## Public API

`eel_governance_legal_audit::transmission::TransmissionEvent` aggregate: `fn verify_legal_basis(&self, rules: &LegalRuleSet) -> ComplianceVerdict`, `fn cite_evidence(&self) -> Vec<EvidenceCitation>`.

Endpoints:
- `GET /v1/governance/legal-audit/transmissions/{id}` - fetch transmission + verdict
- `GET /v1/governance/legal-audit/transmissions/{id}/verdict` - verdict only
- `GET /v1/governance/legal-audit/verdicts?status=violation` - list violations

Verdicts are immutable post-creation; violations trigger automatic transparency-log publishing.

## Feature flags

`#[cfg(feature = "live-rules")]` for Elections Act §39(1C) rules fetched from governance-rules service; tests use embedded rules snapshot.

## Test commands

```bash
cargo test -p eel-governance-legal-audit --all-features
cargo test -p eel-governance-legal-audit --lib section_39_1c_rules
cargo test -p eel-governance-legal-audit --test integration_legal_audit -- --include-ignored
```

Verify all rule combinations (authorized vs unauthorized signatory, in-window vs out-of-window, with/without evidence).

## Bench commands

```bash
cargo bench -p eel-governance-legal-audit verify_10k_transmissions
```

Verification of 10k transmissions should complete in < 500ms (100 us per transmission).

## Dependencies of note

- `uuid6`: transmission and verdict IDs
- `chrono`: election window time checks
- `serde_json`: transmission form parsing
- `tokio`: transparency-svc async calls

## Related docs

- `docs/api-reference.md#legal-audit` (verdict structure)
- Elections Act §39(1C) text (embedded in rule definition comments)
- `migration_006_legal_audit.sql` (schema: transmissions, verdicts, evidence_citations)
