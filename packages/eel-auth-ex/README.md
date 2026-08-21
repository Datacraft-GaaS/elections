# eel-auth-ex

Elixir authentication and authorisation library for IEBC IEMS services.

Provides Keycloak JWT verification (RS256/ES256), a SpiceDB permissions client,
a Phoenix Plug, and a LiveView `on_mount` guard.

## Features

- `Eel.Auth.Subject` - struct representing an authenticated human or workload
- `Eel.Auth.JWKS` - GenServer with ETS-backed JWKS key cache (TTL configurable)
- `Eel.Auth.JWT.verify/2` - signature + claims validation
- `Eel.Auth.SpiceDB` - check_permission and lookup_resources
- `Eel.Auth.Plug` - drop-in Phoenix/Cowboy plug
- `Eel.Auth.LiveAuth` - LiveView `on_mount` guard

## Usage

```elixir
# In application supervisor:
{Finch, name: Eel.Auth.Finch},
{Eel.Auth.JWKS, keycloak_url: "https://auth.example.com", realm: "iems"}

# In Phoenix router pipeline:
plug Eel.Auth.Plug, jwks: Eel.Auth.JWKS

# In a live_session:
live_session :authenticated, on_mount: {Eel.Auth.LiveAuth, :require_authenticated} do
  live "/dashboard", DashboardLive
end
```

## Configuration

```elixir
config :eel_auth,
  spicedb_url: "http://spicedb.internal:50051",
  spicedb_token: "your-token"
```

## Dependencies

- `finch ~> 0.19`
- `jason ~> 1.4`
- `plug ~> 1.17`
- `phoenix_live_view ~> 1.0` (optional)
