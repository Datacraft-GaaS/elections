# iam-seed

Idempotent IAM bootstrap CLI for the eel IEMS. Seeds all four trust-plane components in one run:

- **Keycloak**: creates realms, clients, roles, and mappers from `platform/keycloak/realms/*.json`
- **SpiceDB**: applies `packages/authz-schema/*.zed` schemas via the SpiceDB gRPC write API
- **SPIRE**: registers workload entries from `platform/spire/entries/*.yaml`
- **OpenBao**: writes mount configs and policies from `platform/openbao/`

Wave context: WP-0.3 (trust-plane bootstrap). Runs once per environment at cluster bring-up and again after any schema or realm change.

## Install

```bash
cargo build --release -p iam-seed
# Binary at target/release/iam-seed
```

## Subcommands

| Subcommand | Description |
|---|---|
| `all` | Seed all components in dependency order |
| `keycloak` | Keycloak realms and clients only |
| `spicedb` | SpiceDB schema only |
| `spire` | SPIRE workload entries only |
| `openbao` | OpenBao mounts and policies only |
| `verify` | Check that live state matches declared state (read-only) |

## Usage

```bash
# Full seed (safe to re-run; idempotent):
iam-seed all

# Single component:
iam-seed keycloak --realm-dir platform/keycloak/realms/

# Verify current state without writing:
iam-seed verify

# Verbose output:
iam-seed all --log-level debug
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `KEYCLOAK_URL` | `http://localhost:8080` | Keycloak base URL |
| `KEYCLOAK_ADMIN_USER` | `admin` | Admin username |
| `KEYCLOAK_ADMIN_PASSWORD` | (required) | Admin password |
| `SPICEDB_ENDPOINT` | `localhost:50051` | SpiceDB gRPC endpoint |
| `SPICEDB_PSK` | (required) | SpiceDB pre-shared key |
| `SPIRE_SERVER_SOCKET` | `/tmp/spire-server/private/api.sock` | SPIRE server API socket |
| `OPENBAO_ADDR` | `http://localhost:8200` | OpenBao address |
| `OPENBAO_TOKEN` | (required) | Root or bootstrap token |
| `EEL_REALM_DIR` | `platform/keycloak/realms` | Realm JSON directory |

## Exit codes

- `0`: all components seeded successfully
- `1`: partial failure (one or more components failed; logged individually)
- `2`: missing required environment variable

## Tests

```bash
cargo test -p iam-seed
```

Integration tests spin up Keycloak, SpiceDB, and OpenBao via testcontainers (requires Docker).

## Related docs

- `platform/keycloak/README.md`: realm structure and WebAuthn policy
- `platform/spicedb/README.md`: SpiceDB schema and authz model
- `platform/spire/README.md`: workload identity and attestation
- `platform/openbao/README.md`: secrets management and HSM seal
- `docs/runbooks/iam-bootstrap.md`: first-time IAM bootstrap runbook
