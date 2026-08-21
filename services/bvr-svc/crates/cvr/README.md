# cvr

Module M35 continuous voter register with Elections Act §6A public inspection roll.

Handles continuous voter register (CVR) updates from CRS deltas. Computes anonymized voter count by constituency (Elections Act §6A public inspection, no PII). Generates inspection roll on demand: constituency name, voter count, turnout (post-election). Handles register freeze window (during election) and publishes snapshots for auditing.

Fits into bvr-svc as the inspection layer: tracks register state, serves anonymized public data, supports compliance reporting.

## Public API

Exposed modules via lib.rs:
- cvr_register: CVRRegister, publish_inspection_roll(period) -> InspectionRoll
- inspection_roll: InspectionRoll, by_constituency(), total_voters, anonymized_data_only()
- register_event: VoterRegisterEvent, RegisterSnapshot
- freeze_window: RegisterFreezeWindow, lock_for_election(), unlock_on_close()

## Feature flags

None currently.

## Testing

Unit tests (inspection roll generation):
```bash
cargo test --lib
```

Anonymization tests (verify no PII leakage):
```bash
cargo test inspection_roll::anonymization
```

Freeze window tests (election lockout):
```bash
cargo test freeze_window::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Inspection roll generation (if benches/ exists):
```bash
cargo bench inspection_roll_46_counties
```

## Dependencies of note

Workspace:
- bvr-domain: Voter aggregate
- eel-eventstore-rs: Event subscription
- eel-events-rs: Event schema

External:
- sqlx: PostgreSQL queries (aggregations)

## Related docs

See services/bvr-svc/docs/section-6a-inspection.md for inspection roll spec.
See services/bvr-svc/docs/anonymization-strategy.md for PII handling.
See services/bvr-svc/docs/register-freeze.md for election lockout.
