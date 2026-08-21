# hsm-pkcs11/contract-tests

Contract tests for PKCS#11 HSM adapters.

## Purpose

Verifies key generation, signing, slot enumeration, and vendor-specific slot/label conventions against SoftHSMv2 (dev) and production HSMs.

## Running tests

```bash
cargo test -p eel-contract-tests-hsm-pkcs11
```

Requires SOFTHSM2_CONF=path/to/softhsm2.conf for dev tests.

## What is verified

- Slot enumeration per vendor (Thales Luna 7, Utimaco, SoftHSMv2)
- Slot login with correct PIN
- Key generation for all KeyRole types
- ECDSA-P256 signing with generated keys
- Ed25519 signing for code-signing keys
- Concurrent access via PkcsPool (thread-safe)
- Vendor label discovery (slot label contains expected key_role name)
- Error handling: invalid PIN -> error, not crash
- No raw PIN in logs or debug output

## Dev/test mode

SoftHSMv2 required. Configuration via platform/softhsm/softhsm2.conf:

```
directories.tokendir = /tmp/softhsm
library = /usr/lib/softhsm/libsofthsm2.so
```

## Production verification

On production HSM (Thales Luna 7 or Utimaco), run with --features integration-tests (manual).
