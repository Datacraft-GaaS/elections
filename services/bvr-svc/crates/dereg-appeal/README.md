# dereg-appeal

Module M36 voter deregistration appeals with ODPC-compliant right-to-erasure workflows.

Handles voter deregistration appeals (voter requests erasure, challenges deregistration). Implements ODPC (Office of the Data Protection Commissioner, Kenya DPA 2019) right-to-erasure workflows: appeal submission, adjudication window (30 days), the Commission decision, erasure confirmation. Tracks appeal lifecycle and maintains audit trail. Emits bvr.deregistration.appealed event.

Fits into bvr-svc as the erasure request handler: voters file appeals, the Commission adjudicates, domain executes erasure with full audit.

## Public API

Exposed modules via lib.rs:
- appeal_case: DeregistrationCase, file_appeal(voter_id, reason) -> CaseId
- adjudication: Adjudication, adjudicate_case(case_id, decision) -> AdjudicationResult
- erasure: Erasure, execute_erasure(voter_id) -> ErasureProof
- audit: ErasureAuditLog, log_erasure(voter_id, case_id, decision, timestamp)

## Feature flags

None currently.

## Testing

Unit tests (appeal state machine):
```bash
cargo test --lib
```

Adjudication workflow tests:
```bash
cargo test adjudication::
```

Erasure audit trail tests:
```bash
cargo test erasure::audit
```

Integration tests (ODPC compliance):
```bash
cargo nextest run --features odpc-compliance
```

## Benchmarks

Erasure execution latency (if benches/ exists):
```bash
cargo bench erasure_execution
```

## Dependencies of note

Workspace:
- bvr-domain: Voter erasure
- eel-events-rs: Appeal event emission
- eel-crypto-rs: Audit trail hashing

External:
- sqlx: PostgreSQL persistence

## Related docs

See services/bvr-svc/docs/deregistration-appeal.md for appeal procedures.
See services/bvr-svc/docs/odpc-erasure-compliance.md for ODPC requirements.
See services/bvr-svc/docs/right-to-erasure.md for Kenya DPA 2019 §40.
See docs/runbooks/deregistration-appeal-adjudication.md for operational guide.
