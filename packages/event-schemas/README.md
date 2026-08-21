# event-schemas

JSON Schema files for every CloudEvent type published by the eel platform, organized by `(context, aggregate, event, version)`.

## Purpose in context

Every event in the system has a schema registered in Karapace and stored here as a canonical reference. Producers validate their payloads against the schema before publishing (via `KarapaceClient` in `eel-events-*`). Consumers validate received payloads. Schema evolution follows additive-only backward-compatible rules; breaking changes require a new version directory.

## Layout

```
event-schemas/
  rts/
    form/
      form34a_captured/
        v1.schema.json
      form34b_captured/
        v1.schema.json
    merkle/
      merkle_root_computed/
        v1.schema.json
    session/
      session_opened/
        v1.schema.json
    snapshot/
    token/
  bvr/
    credential/
    declaration/
    ...
  transparency/
  # ... one directory per bounded context
  _shared/
    hlc.schema.json          # HLC string format
    envelope.schema.json     # base CloudEvent envelope
    uuid7.schema.json        # UUIDv7 pattern
```

## Schema naming convention

```
event-schemas/<context>/<aggregate>/<event_name>/v<n>.schema.json
```

Maps to CloudEvent `type`:

```
eel.<context>.<aggregate>_<event_name>.v<n>
# e.g.: eel.rts.form_form34a_captured.v1
```

## Regenerating Karapace registrations

```bash
# Register all schemas in the local Karapace instance:
just register-schemas

# Register a single context:
just register-schemas --context rts

# Validate all schemas are syntactically correct JSON Schema draft-07:
just validate-schemas
```

## Verify

```bash
# Check schema registry is in sync with this directory:
just check-schemas-registered

# Validate a payload against a schema manually:
ajv validate -s event-schemas/rts/form/form34a_captured/v1.schema.json \
             -d /path/to/payload.json
```

## Adding a new event

1. Create `event-schemas/<ctx>/<aggregate>/<event>/v1.schema.json`
2. Run `just validate-schemas` to confirm it parses
3. Register in Karapace: `just register-schemas --context <ctx>`
4. Add the `$id` value to `packages/eel-events-rs/src/karapace.rs` schema allow-list
5. Write a test in the owning service's `tests/ci/` directory

## Related docs

- `docs/EVENT-CONVENTIONS.md` - topic naming, envelope fields, evolution rules
- `packages/openapi-contracts/README.md` - REST API specs (separate from events)
- `packages/eel-events-rs/README.md` - producer/consumer that validates against these schemas

## Limitations

JSON Schema validation is advisory in the consumer path; the consumer logs a warning on schema mismatch rather than rejecting the message (to avoid consumer stall). Producers are required to pass validation before publishing. Schema evolution beyond backward-compatible additions requires a new version directory and a migration plan reviewed by the architecture team.
