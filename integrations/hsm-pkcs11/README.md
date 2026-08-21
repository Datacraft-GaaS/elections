# integrations/hsm-pkcs11

I07 HSM PKCS#11 vendor profiles.

Provides `HsmProfile::new(vendor: Vendor, role: KeyRole)` returning a
`PkcsPool` with vendor-specific slot/label conventions.

## Vendors

| Vendor | Enum | Library |
|---|---|---|
| Thales Luna 7 | `ThalesLuna7` | `/usr/lib/libCryptoki2_64.so` |
| Utimaco CryptoServer | `UtimacoOpenusr` | `/usr/lib/libcs_pkcs11_R3.so` |
| SoftHSMv2 (dev) | `SoftHsm2` | `/usr/lib/softhsm/libsofthsm2.so` |

## Key roles

`Sth`, `Declaration`, `KitCa`, `CredentialCa`, `CodeSigning`, `CycleKek`

## No double needed

SoftHSMv2 is the dev/test double. Configure with
`SOFTHSM2_CONF=platform/softhsm/softhsm2.conf`.

## Contract tests

Run: `cargo test -p eel-adapter-hsm-pkcs11-contract-tests`
