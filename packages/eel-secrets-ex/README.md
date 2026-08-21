# eel-secrets-ex

OpenBao (Vault-compatible) secrets client for eel Elixir services: dynamic Postgres credentials, transit encryption, and PKI leaf cert issuance. Mirror of `eel-secrets-rs`.

## Purpose in context

Elixir services obtain short-lived Postgres credentials from OpenBao at startup via `Eel.Secrets.PgDynamicCred`, a GenServer that auto-renews leases. Transit encryption wraps sensitive fields before storage.

## Public modules

| Module | Description |
|---|---|
| `Eel.Secrets.Client` | Base Finch-backed HTTP client for OpenBao API |
| `Eel.Secrets.PgDynamicCred` | GenServer; issues and auto-renews Postgres leases |
| `Eel.Secrets.Transit` | `encrypt/2`, `decrypt/2` via OpenBao transit engine |
| `Eel.Secrets.Pki` | `issue_leaf_cert/2` - mTLS leaf cert issuance |

## Layout

```
eel-secrets-ex/
  lib/
    eel_secrets.ex
    eel_secrets/
      client.ex
      pg_dynamic_cred.ex
      transit.ex
      pki.ex
  test/
  mix.exs
```

## Usage

```elixir
# application.ex supervision tree
children = [
  {Eel.Secrets.PgDynamicCred,
   name: :pg_cred,
   addr: System.get_env("OPENBAO_ADDR"),
   token: System.get_env("OPENBAO_TOKEN"),
   role_path: "database/creds/portal-backend"},
]

# Get DSN for Ecto repo:
dsn = Eel.Secrets.PgDynamicCred.dsn(:pg_cred)
# => "postgresql://dynamic-user:dynamic-pass@db:5432/portal"

# Transit encrypt:
transit = Eel.Secrets.Transit.new(addr, token, "eel-data")
{:ok, cipher} = Eel.Secrets.Transit.encrypt(transit, "voter-id-plaintext")
{:ok, plain}  = Eel.Secrets.Transit.decrypt(transit, cipher)
```

## Config / env

| Variable | Description |
|---|---|
| `OPENBAO_ADDR` | OpenBao server address |
| `OPENBAO_TOKEN` | Service token |
| `OPENBAO_TRANSIT_KEY` | Transit key name |

## Dependencies

`finch`, `jason`.

## Install

```elixir
{:eel_secrets, path: "../../packages/eel-secrets-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-secrets-rs/README.md` - Rust canonical implementation
- `docs/adr/0007-secrets.md` - OpenBao role design and lease policies

## Limitations

`PgDynamicCred` renewal is handled by an OTP timer inside the GenServer. If the GenServer crashes and the supervisor restarts it, a new lease is issued. The Ecto repo must be reconfigured with the new DSN; configure Ecto to call `PgDynamicCred.dsn/1` dynamically rather than reading it at compile time.
