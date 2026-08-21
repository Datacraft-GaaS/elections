# SoftHSMv2 for eel local dev

SoftHSMv2 provides a software PKCS#11 token that mimics the prod HSM layout (Thales Luna 7 or Utimaco CryptoServer) during local development and CI.

## Slot layout

| Slot label | Purpose |
| eel-sth | Signed Tree Head (transparency log) signing key |
| eel-declaration | Result declaration signing key |
| eel-kit-ca | Kit provisioning CA private key |
| eel-credential-ca | Credential issuance CA private key |
| eel-code-signing | Artifact code-signing key |
| eel-cycle-kek | Cycle Key Encryption Key (transit encryption) |

## PKCS#11 module paths

| Platform | Path |
| Linux (apt/rpm) | `/usr/lib/softhsm/libsofthsm2.so` |
| Linux (Nix) | `$(nix-build '<nixpkgs>' -A softhsm --no-out-link)/lib/softhsm/libsofthsm2.so` |
| macOS (Homebrew) | `/usr/local/lib/softhsm/libsofthsm2.so` or `/opt/homebrew/lib/softhsm/libsofthsm2.so` |

Set `PKCS11_MODULE_PATH` to the path above before running `hsm-init`.

## Setup

```
bash tools/softhsm/init.sh
cargo build --release -p hsm-init
cargo run --release -p hsm-init -- init-slots
```

## Environment variables

| Variable | Default | Purpose |
| `SOFTHSM2_CONF` | `~/.softhsm2/softhsm2.conf` | SoftHSM config file |
| `SOFTHSM_TOKEN_DIR` | `~/.softhsm2/tokens` | Token storage directory |
| `PKCS11_MODULE_PATH` | (platform-specific) | Path to `libsofthsm2.so` |
