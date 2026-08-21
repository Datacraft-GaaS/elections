# hello_app (Phoenix app)

Hello-world Phoenix/Elixir app used as scaffold template and CI smoke-test.

## Purpose

Scaffolding template and CI validation for the Elixir/Phoenix tier convention
(`services/<svc>/ex/<app>/`). Not a production application. Demonstrates minimum
viable Phoenix app with Ecto, Broadway stub, and Oban stub wired to the eel base
configuration.

## Mix Commands

```bash
cd services/hello-svc/ex/hello_app
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `SECRET_KEY_BASE` | Phoenix secret key |

## Notes

- Template only. Do not deploy to production Kubernetes.
- Used for CI validation of hello-svc and Elixir package changes.
