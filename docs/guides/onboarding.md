# New Engineer Onboarding

## Prerequisites

- Nix 2.18+ with flakes enabled (`experimental-features = nix-command flakes` in `~/.config/nix/nix.conf`)
- Docker Desktop or `podman` (for service integration tests)
- `just` task runner (`nix profile install nixpkgs#just` or via the dev shell)
- Access to the internal container registry and secrets (request from your team lead)

## Step 1: Enter the development shell

```sh
git clone <repo-url> eel
cd eel
nix develop
```

The flake provides: Rust toolchain (pinned), Python 3.12 + uv, Elixir + Mix, Dart + Flutter, `just`, `openapi-generator-cli`, `yq`, `jq`, `k3d`, `helm`, `kubectl`, `sqlx-cli`, `pyright`, `cargo-nextest`.

## Step 2: Start local services

```sh
just up
```

This starts PostgreSQL, Kafka (Redpanda), Redis, MinIO, Keycloak, and SPIRE via Docker Compose. Health checks pass within 60 s on a 2020-era laptop.

## Step 3: Run the CI test suite

```sh
just test
```

Runs `uv run pytest -vxs tests/ci` for Python services and `cargo nextest run` for Rust services. All tests in `tests/ci/` must pass before pushing.

## Step 4: First-service walkthrough

Pick `rts-svc` as a representative Rust service and `bvr-svc` as the scale anchor.

```sh
# Rust service (rts-svc)
cd services/rts-svc
cargo build
cargo nextest run

# Python service (boundary-svc uses Django)
cd services/boundary-svc
uv run python manage.py migrate
uv run pytest -vxs tests/ci
```

Explore `services/rts-svc/src/` to see the standard layout:
- `main.rs` - startup, config, DI wiring
- `service.rs` - domain logic (primary module)
- `views.rs` - Pydantic-equivalent Rust structs (serde)
- `routes.rs` - Axum route definitions
- `events.rs` - CloudEvents outbox helpers

## Step 5: Understand the event bus

Read `docs/EVENT-CONVENTIONS.md` and `packages/event-schemas/README.md`. Run a local Kafka consumer to watch events:

```sh
just kafka-console bvi
# or
rpk topic consume eel.rts --brokers localhost:9092
```

## Step 6: Key documents to read first

- `docs/adr/README.md` - 23 ADRs covering all major decisions
- `docs/architecture/README.md` - service map and shared packages
- `docs/guides/contribution.md` - commit style and PR checklist
- `docs/plans/00-principles.md` - design principles
- `tender.txt` in the repo root - the binding RFP; all functional requirements trace here

## Secrets and credentials

Local secrets are managed via OpenBao (HashiCorp Vault fork). Run `just secrets-init` after `just up` to seed the local dev vault with test values. Never commit secrets; the pre-commit hook checks for them.

## Troubleshooting

- Kafka not reachable: `just down && just up` to reset volumes.
- PostgreSQL migration fails: `just db-reset` drops and recreates the dev database.
- SPIRE SVID errors: `just spire-reset` restarts the SPIRE server and agents.
- Type check errors: run `uv run pyright` (Python) or `cargo check` (Rust).
