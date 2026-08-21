# eel-pii-ex

PII handling with data class tiers and optional Ecto types for IEBC IEMS. Implements ADR-0016.

## Data classes

| Class | Tier | Examples |
|-------|------|----------|
| :A | Restricted | biometric, national ID |
| :B | Restricted | financial, health records |
| :C | Sensitive | contact details |
| :D | Sensitive | name, address |
| :E | Public | publicly available info |

Values wrapped in PII structs never leak through `String.Chars` or `Inspect` — they always return a redaction string. Call `reveal/2` with a stated purpose to access the raw value; this emits an audit log entry via `Logger.info/2`.

## Installation

```elixir
# mix.exs
{:eel_pii, path: "../eel-pii-ex"}

# With Ecto support
{:eel_pii, path: "../eel-pii-ex"},
{:ecto, "~> 3.12"}
```

## Usage

```elixir
alias Eel.PII

# Wrap values
national_id = PII.restricted("KE-12345678", :A)
email       = PII.sensitive("voter@example.com", :C)
county      = PII.public("Nairobi")

# Safe display — always redacted
IO.puts(national_id)  # => [REDACTED:A]
inspect(email)        # => [REDACTED:C]

# Reveal with audit trail
{:ok, raw} = PII.reveal(national_id, "identity-verification")

# Class and redaction
PII.class(national_id)   # => :A
PII.redact(national_id)  # => "[REDACTED:A]"
```

### Ecto types (optional)

```elixir
field :email, Eel.PII.EctoType.Sensitive
field :county, Eel.PII.EctoType.Public

# Restricted types refuse dump/1 to prevent accidental raw storage
```

## Testing

```sh
mix test
```
