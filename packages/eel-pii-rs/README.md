# eel-pii-rs

PII classification wrappers enforcing the five-tier data-class model (ADR-0016) at the type level.

## Purpose in context

Any field holding personal data must be wrapped in the appropriate tier type. The type system prevents accidental logging or serialization of restricted data, and every `.reveal()` call on a Tier-A or Tier-B value is recorded in the audit trail. This crate is a compile-time privacy guardrail, not a runtime encryption engine.

## Public types

| Type | Tier | Serde | Access |
|---|---|---|---|
| `PIIRestricted<T>` | A (biometric, national ID) | compile-error if serialized directly | `.reveal(purpose, sink)` |
| `PIISensitive<T>` | B (name, DOB, address) | compile-error if serialized directly | `.reveal(purpose, sink)` |
| `PIIPublic<T>` | C (polling station name) | transparent | `.value()` |
| `PIIOperational<T>` | D (internal reference) | transparent | `.value()` |
| `PIIEphemeral<T>` | E (session tokens, nonces) | transparent | `.value()` |

## Key traits

- `PIIAuditSink` - receives `(subject, purpose, field_name, tier)` on every reveal
- `NoopAuditSink` - test / placeholder implementation

## Layout

```
eel-pii-rs/
  src/lib.rs    # all types in one file
  Cargo.toml
```

## Usage

```rust
use eel_pii_rs::{PIIRestricted, PIISensitive, Purpose, NoopAuditSink};

struct Voter {
    national_id: PIIRestricted<String>,
    full_name:   PIISensitive<String>,
}

let voter = Voter {
    national_id: PIIRestricted::new("KE12345678".to_string()),
    full_name:   PIISensitive::new("Jane Mwangi".to_string()),
};

let sink = NoopAuditSink;
let id = voter.national_id.reveal(Purpose::Identification, &sink);
// voter.national_id.to_string() => "[RESTRICTED]"
```

## Features

| Feature | What it adds |
|---|---|
| `sqlx-postgres` (default) | `sqlx::Type` impls so wrappers map to TEXT columns |

## Dependencies

`serde`, `thiserror`, `tracing`. `sqlx` optional.

## Build

```bash
cargo build -p eel-pii-rs
```

## Test

```bash
cargo nextest run -p eel-pii-rs
# proptest and trybuild tests verify compile-time guarantees:
cargo test -p eel-pii-rs
```

## Related docs

- `docs/adr/0016-data-classification.md` - tier definitions and retention rules
- `packages/eel-observability-rs/README.md` - PII log scrubber built on this crate
- `packages/data-dictionary/README.md` - column classification per table

## Limitations

The compile-time protection only covers Rust code. Python and Elixir mirrors enforce the contract at runtime via Pydantic validators and guard functions. Column-level encryption for Tier-A data is implemented at the database layer, not inside these wrappers.
