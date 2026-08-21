# special-voter

Special voter workflows crate for M25. Manages voter identification and event recording at special facilities: Kenya Prisons, KDF/GSU bases, hospital wards, IDP camps, UNHCR refugee stations, and diaspora-adjacent facilities not covered by M24.

## What it does

Some voters cannot access their registered polling station on election day. M25 provides facility-specific enrolment and identification workflows, coordinating with Kenya Prisons, Kenya Defence Forces, Kenya Prisons, UNHCR ProGres v4, NDOC, Red Cross, and county health authorities. Each special facility is pre-registered with a geo-fence; identification events are signed and routed to bvi-svc as if from a standard kit, so turnout rollups remain consistent. IDP voters may be reassigned to their displacement location via a structured `IDPReassignment` event tracked against M35 CVR.

## Public API surface

- `mod facility` - `SpecialVoterFacility`, `FacilityType`, `register_facility()`, `list_facilities()`
- `mod event` - `SpecialVoterEvent`, `record_identification()`, `list_events_by_facility()`
- `mod idp` - `IDPReassignment`, `reassign_voter()`, confirm ODPC-compliant data minimisation
- `mod sync` - Offline sync protocol for facility kits (7-day queue, same protocol as biometric-edge-svc)

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `prisons-api` | off | Kenya Prisons roster API integration |
| `unhcr-progres` | off | UNHCR ProGres v4 verification |

## Test commands

```bash
cargo nextest run -p special-voter
cargo nextest run -p special-voter --test integration -- --include-tag special-voter
```

## Dependencies of note

- `eel-events-rs` (workspace): event envelope, HLC, outbox
- `eel-pii-rs` (workspace): PIIRestricted handling for facility rosters
- `eel-resilience-rs` (workspace): circuit breaker for government API adapters
- `biometric-edge-svc` crates: shares kit-attest patterns for facility kit attestation

## Related docs

- `docs/plans/modules/M25-special-voter-workflows.md`
- Elections Act §44A complementary mechanism
- `packages/event-schemas/support/special-voter.*.json`
- `docs/runbooks/support-admin-svc-degradation.md`
