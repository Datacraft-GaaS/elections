# support-api

Field support ticketing and remote diagnostics API for M32 field support staff.

Manages field support tickets (device issues, network problems, voter disputes), remote diagnostic captures (logs, screenshots, device state), and kit dispatch requests. Integrates with inventory-svc for device provisioning and location-svc for field team GPS tracking. Provides ticket escalation workflow and SLA tracking for ECC field coordination.

Fits into field-mobile-svc as the support infrastructure for M32 field support staff: field operations can file tickets for on-station issues, attach diagnostics, request remote support, and escalate to ECC for urgent coordination.

## Public API

Exposed modules via lib.rs:
- ticketing: TicketService, create_ticket(location, category, description) -> TicketId
- diagnostics: DiagnosticCapture, capture_device_state(device_id) -> StateSnapshot
- escalation: EscalationWorkflow, escalate_ticket(ticket_id, reason) -> EscalationResult
- query: TicketQuery, list_open_tickets(station_id) -> TicketList

## Feature flags

None currently defined. Full ticketing workflow always enabled.

## Testing

Unit tests:
```bash
cargo test --lib
```

Ticket lifecycle tests:
```bash
cargo test ticketing::tests
```

Escalation workflow tests:
```bash
cargo test escalation::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires PostgreSQL):
```bash
cargo test --test '*integration*' -- --ignored
```

## Benchmarks

Ticket creation latency:
```bash
cargo bench --bench ticket_creation
```

Diagnostic capture and compression:
```bash
cargo bench --bench diagnostic_capture
```

## Dependencies of note

Workspace:
- eel-auth-rs: Field staff authentication
- eel-observability-rs: Tracing and metrics

External:
- axum: Async HTTP server
- tokio: Async runtime
- sqlx: PostgreSQL ticket and diagnostic storage
- serde: Serialization for diagnostics

## Related docs

See services/field-mobile-svc/docs/support-ticket-workflow.md for ticket lifecycle.
See docs/runbooks/support-api-degradation.md for operational runbook.
