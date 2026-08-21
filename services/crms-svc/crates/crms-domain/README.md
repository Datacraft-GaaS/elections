# crms-domain

Core domain aggregates and events for candidate nomination, party registration, and ballot proof management.

Defines Candidate, Nomination, ClearanceDecision, BallotProof aggregates with state machines. Models PoliticalParty, CoalitionRegistration, and SymbolSubmission for party lifecycle. Emits domain events (nomination.submitted, nomination.cleared, nomination.rejected, party.registered, symbol.approved). Enforces quota rules and candidate eligibility constraints. Provides idempotent command handling for nomination and party operations.

Fits into crms-svc as the business logic layer: crms-api uses crms-domain to validate and apply commands, and publishes resulting events to Kafka.

## Public API

Exposed modules via lib.rs:
- nomination: Nomination (aggregate), NominationCommand enum
- clearance: ClearanceDecision (aggregate), decision_state machine
- ballot: BallotProof (aggregate), PDF metadata and verification
- party: PoliticalParty (aggregate), PartyCommand enum
- symbol: SymbolSubmission (aggregate), Pantone validation
- events: DomainEvent enum (all nomination, party, symbol events)
- commands: CommandHandler trait for idempotent processing
- rules: QuotaRules, CandidateEligibility validation

## Feature flags

None currently defined. Full state machines and validation always enabled.

## Testing

Unit tests:
```bash
cargo test --lib
```

Nomination state machine tests:
```bash
cargo test nomination::state_machine_tests
```

Clearance decision logic tests:
```bash
cargo test clearance::tests
```

Quota validation tests:
```bash
cargo test rules::quota_tests
```

Party registration state machine tests:
```bash
cargo test party::state_machine_tests
```

Symbol validation tests (Pantone checks):
```bash
cargo test symbol::validation_tests
```

Run all with:
```bash
cargo nextest run
```

Property tests (state coverage):
```bash
cargo test --lib -- --ignored  # proptest scenarios
```

Idempotence tests (duplicate commands):
```bash
cargo test commands::idempotence_tests
```

## Benchmarks

Command application and event emission:
```bash
cargo bench --bench command_handling
```

Quota validation:
```bash
cargo bench --bench quota_validation
```

Candidate eligibility checking:
```bash
cargo bench --bench eligibility_check
```

## Dependencies of note

Workspace:
- eel-events-rs: Event publishing and schema
- eel-observability-rs: Tracing

External:
- serde: Event serialization
- thiserror: Error types
- uuid: Event IDs (UUID7)
- regex: Quota expression parsing

## Related docs

See services/crms-svc/docs/nomination-state-machine.md for state diagram.
See services/crms-svc/docs/quota-rules.md for quota calculation logic.
See services/crms-svc/docs/symbol-pantone-validation.md for colour registry.
See docs/event-schema/crms-events.yaml for event definitions.
