# eel-auth-ex

JWT/JWKS verification, SpiceDB ReBAC authorization, and audit emission for eel Elixir services. Mirror of `eel-auth-rs`.

## Purpose in context

Phoenix services validate bearer tokens and check SpiceDB permissions through this library. It provides a Plug that sets `conn.assigns.eel_subject`, a SpiceDB client with fail-closed semantics, and an `AuditSink` behaviour. A `require_permission` Plug builder covers common route-level authorization patterns.

## Public modules

| Module | Description |
|---|---|
| `Eel.Auth.JwksCache` | GenServer; caches JWKS keys with TTL and kid-miss refetch |
| `Eel.Auth.Subject` | Struct: `sub`, `roles`, `scopes`, `is_service` |
| `Eel.Auth.Plug` | Plug; sets `conn.assigns.eel_subject` or halts with 401 |
| `Eel.Auth.RequirePermission` | Plug builder factory: `require_permission("rts", resource, action)` |
| `Eel.Auth.SpiceDB` | GenServer client; `check/4 -> :allow | :deny`; fail-closed |
| `Eel.Auth.AuditSink` | Behaviour; `record/4` |

## Layout

```
eel-auth-ex/
  lib/
    eel_auth.ex
    eel_auth/
      jwks_cache.ex
      subject.ex
      plug.ex
      require_permission.ex
      spice_db.ex
      audit_sink.ex
  test/
  mix.exs
```

## Usage

```elixir
# router.ex
pipeline :api do
  plug Eel.Auth.Plug
  plug Eel.Auth.RequirePermission, context: "rts", action: "submit"
end
```

```elixir
# application.ex supervision tree
children = [
  {Eel.Auth.JwksCache, jwks_url: System.get_env("KEYCLOAK_JWKS_URL")},
  {Eel.Auth.SpiceDB, endpoint: System.get_env("SPICEDB_ENDPOINT"),
                     key: System.get_env("SPICEDB_PRESHARED_KEY")},
]
```

## Config / env

| Variable | Description |
|---|---|
| `KEYCLOAK_JWKS_URL` | JWKS endpoint |
| `SPICEDB_ENDPOINT` | SpiceDB gRPC address |
| `SPICEDB_PRESHARED_KEY` | SpiceDB preshared key |

## Dependencies

`finch`, `jason`, `plug`.

## Install

```elixir
{:eel_auth, path: "../../packages/eel-auth-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
mix dialyzer
```

## Related docs

- `packages/authz-schema/README.md` - SpiceDB schema per bounded context
- `packages/eel-auth-rs/README.md` - Rust mirror
- `docs/adr/0004-iam.md` - Keycloak realm and scope taxonomy

## Limitations

`Eel.Auth.SpiceDB` communicates with SpiceDB over HTTP/2 via Finch. Any connection error causes `:deny`. Include SpiceDB in the service's readiness check to prevent traffic reaching the service while SpiceDB is unreachable.
