# hsm-init

Bootstrap PKCS#11 keys in SoftHSM slots for eel local development and CI. Mirrors the production HSM slot layout so services run against the same key labels without a physical HSM.

Wave context: WP-0.1 (toolchain). Run once per developer workstation and once per CI worker image build.

## Prerequisites

- SoftHSM2 installed and initialized via `tools/softhsm/init.sh`
- `PKCS11_MODULE_PATH` pointing to `libsofthsm2.so`

## Install

```bash
# SoftHSM2 first:
bash tools/softhsm/init.sh

# Then build and run:
cargo build --release -p hsm-init
./target/release/hsm-init init-slots
```

## Subcommands

| Subcommand | Description |
|---|---|
| `init-slots` | Create all slots and generate keys per the standard layout |
| `list` | List existing slots and key labels |
| `destroy` | Wipe all eel slots (destructive, dev only) |
| `verify` | Check that all expected slots and keys exist |

## Usage

```bash
# Initialize all slots:
hsm-init init-slots

# Verify slot layout after init:
hsm-init verify

# List slots:
hsm-init list

# Destroy and re-initialize (dev reset):
hsm-init destroy && hsm-init init-slots
```

## Slot layout

| Label | Key type | Purpose |
|---|---|---|
| `eel-sth` | Ed25519 | Signed Tree Head (transparency log) |
| `eel-declaration` | Ed25519 | Result declaration signing |
| `eel-kit-ca` | P-256 | Kit provisioning CA |
| `eel-credential-ca` | P-256 | Credential issuance CA |
| `eel-code-signing` | Ed25519 | Artifact code signing |
| `eel-cycle-kek` | AES-256 | Cycle Key Encryption Key |

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `PKCS11_MODULE_PATH` | (platform-specific) | Path to `libsofthsm2.so` |
| `SOFTHSM2_CONF` | `~/.softhsm2/softhsm2.conf` | SoftHSM2 config file |
| `EEL_HSM_PIN` | `1234` | PKCS#11 user PIN |
| `EEL_HSM_SO_PIN` | `12345678` | PKCS#11 security officer PIN |

## Exit codes

- `0`: success
- `1`: slot already initialized (non-fatal; re-run is safe)
- `2`: module not found or PIN error

## Related docs

- `tools/softhsm/README.md`: SoftHSM2 wrapper and module paths
- `platform/hsm-signer/README.md`: production HSM signer deployment
- `docs/runbooks/hsm-ceremony.md`: key ceremony runbook for production key rotation
