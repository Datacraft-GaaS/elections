# eel-pii-ex

PII classification wrappers and Ecto types for eel Elixir services, enforcing the ADR-0016 five-tier data model. Mirror of `eel-pii-rs`.

## Purpose in context

Elixir services handling voter records wrap personal data fields in typed PII structs. The Ecto custom types prevent accidental serialization of Tier-A/B values, and every `reveal/3` call emits an audit record. Column-level encryption for Tier-A fields is enforced at the Postgres layer; these types provide the application guardrail.

## Public modules

| Module | Description |
|---|---|
| `Eel.PII` | `Purpose` enum, `reveal/3` function |
| `Eel.PII.Restricted` | Tier A struct + Ecto type; `new/1`, `reveal/3`, `to_string/1` returns `"[RESTRICTED]"` |
| `Eel.PII.Sensitive` | Tier B struct + Ecto type |
| `Eel.PII.Public` | Tier C struct + Ecto type; transparent serialization |
| `Eel.PII.Operational` | Tier D struct + Ecto type |
| `Eel.PII.Ephemeral` | Tier E struct |
| `Eel.PII.AuditSink` | Behaviour for audit record emission |

## Layout

```
eel-pii-ex/
  lib/
    eel_pii.ex
    eel_pii/
      restricted.ex
      sensitive.ex
      public.ex
      operational.ex
      ephemeral.ex
      audit_sink.ex
  test/
  mix.exs
```

## Usage

```elixir
alias Eel.PII.{Restricted, Sensitive, Purpose}

voter = %{
  national_id: Restricted.new("KE12345678"),
  full_name:   Sensitive.new("Jane Mwangi"),
}

# Inspect (safe):
IO.inspect(voter.national_id)  # %Eel.PII.Restricted{value: "[RESTRICTED]"}

# Reveal with audit:
nid = Restricted.reveal(voter.national_id, Purpose.identification(), audit_sink)
# => "KE12345678"
```

## Ecto schema usage

```elixir
schema "voters" do
  field :national_id, Eel.PII.Restricted
  field :full_name,   Eel.PII.Sensitive
end
```

## Dependencies

`ecto` optional.

## Install

```elixir
{:eel_pii, path: "../../packages/eel-pii-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-pii-rs/README.md` - canonical Rust implementation
- `packages/data-dictionary/README.md` - column classification per table
- `docs/adr/0016-data-classification.md` - tier definitions

## Limitations

The Ecto type stores values as plaintext TEXT in Postgres. Tier-A column-level encryption is a DBA-level concern; the application type provides only the access-control guardrail.
