# hsm-pkcs11/adapter

Rust crate wrapping PKCS#11 vendor HSM libraries (Thales Luna 7, Utimaco, SoftHSMv2).

## Layout

- `src/lib.rs` - public API (HsmProfile, PkcsPool, Vendor enum, KeyRole enum)
- `src/thales.rs` - Thales Luna 7 vendor-specific slot/label handling
- `src/utimaco.rs` - Utimaco CryptoServer vendor-specific handling
- `src/softhsm.rs` - SoftHSMv2 dev/test support

## Key types

HsmProfile: vendor + key_role -> PkcsPool (connection pool).

Vendor: ThalesLuna7 | UtimacoOpenusr | SoftHsm2.

KeyRole: Sth (state transmission), Declaration, KitCa, CredentialCa, CodeSigning, CycleKek.

PkcsPool: manages slot login, PIN entry, connection pooling.

## Build

```bash
cargo build -p eel-adapter-hsm-pkcs11-rs --release
```

## Environment variables

- SOFTHSM2_CONF: path to softhsm2.conf (dev/test only)
- HSM_SLOT_LABEL: override slot label discovery (optional)
- HSM_SLOT_PIN: PIN for slot login (from vault, not committing to .env)
- HSM_VENDOR: Vendor override (default: detect via library path)

## Testing

```bash
cargo test -p eel-adapter-hsm-pkcs11
```

Requires SOFTHSM2_CONF set for unit tests.

## Production

Production uses Thales Luna 7 or Utimaco. Library path configured via systemd service.
