# eel-secrets-ex

Elixir OpenBao (Vault-compatible) secrets client for IEBC IEMS services.

Provides KV v2 reads, Transit encrypt/decrypt, and automatic PostgreSQL
dynamic credential rotation.

## Features

- `Eel.Secrets.read/2` - KV v2 secret reads
- `Eel.Secrets.encrypt/2` and `Eel.Secrets.decrypt/2` - Transit engine
- `Eel.Secrets.PgDynamicCred` - GenServer with automatic lease renewal
- `Eel.Secrets.JITAdminCred` - just-in-time short-lived admin credentials

## Usage

```elixir
# In application supervisor:
Eel.Secrets.child_spec([])

# Read a secret:
{:ok, data} = Eel.Secrets.read("myapp/db")

# Encrypt/decrypt via Transit:
{:ok, ct} = Eel.Secrets.encrypt("secret", "my-key")
{:ok, pt} = Eel.Secrets.decrypt(ct, "my-key")

# Dynamic PG creds (auto-renewed):
{Eel.Secrets.PgDynamicCred, role: "app_role"}
{:ok, %{username: u, password: p}} = Eel.Secrets.PgDynamicCred.get_credential()
```

## Configuration

```elixir
config :eel_secrets,
  bao_url: "http://vault.internal:8200",
  token: "your-openbao-token"
```

## Dependencies

- `finch ~> 0.19`
- `jason ~> 1.4`
