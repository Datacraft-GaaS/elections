# vprs-domain

Core domain aggregates and events for voter registration application lifecycle.

Defines RegistrationApplication aggregate with state machine (submitted, iprs-verified, approved, rejected, updated). Models ChangeRequest for continuous register changes. Emits domain events (application.submitted, application.iprs-verified, application.approved, application.rejected). Provides idempotent command handling for registration operations.

Fits into vprs-svc as the business logic layer: vprs-api uses vprs-domain to validate and apply commands, and publishes resulting events to Kafka.

## Public API

Exposed modules via lib.rs:
- application: RegistrationApplication (aggregate), ApplicationCommand enum
- events: ApplicationEvent enum (submitted, iprs-verified, approved, rejected, updated)
- change_request: ChangeRequest aggregate
- commands: CommandHandler trait for idempotent processing

## Feature flags

None currently defined. Full state machine always enabled.

## Testing

Unit tests:
```bash
cargo test --lib
```

State machine tests (all valid transitions):
```bash
cargo test application::state_machine_tests
```

Event emission tests:
```bash
cargo test events::tests
```

Idempotence tests (duplicate commands):
```bash
cargo test commands::idempotence_tests
```

Run all with:
```bash
cargo nextest run
```

Property tests (state machine coverage):
```bash
cargo test --lib -- --ignored  # proptest scenarios
```

## Benchmarks

Command application and event emission:
```bash
cargo bench --bench command_handling
```

## Dependencies of note

Workspace:
- eel-events-rs: Event publishing and schema
- eel-observability-rs: Tracing

External:
- serde: Event serialization
- thiserror: Error types
- uuid: Event IDs (UUID7)

## Related docs

See services/vprs-svc/docs/application-state-machine.md for state diagram.
See docs/event-schema/vprs-events.yaml for event definitions.
