# How to build all mobile APKs

This document explains how to use `scripts/mobile/build-all-apks.sh` to build every deployable APK in the eel repository.

## Quick start

```bash
# Build everything (debug APKs, fastest).
scripts/mobile/build-all-apks.sh

# Build everything with gates (analyse + tests must pass).
scripts/mobile/build-all-apks.sh --release --analyze --test

# Skip the two M3 hardware-gated apps (no FAP60 / AntIris USB devices).
scripts/mobile/build-all-apks.sh --skip=bvr-kit,bvi-kit
```

## What gets built

14 Flutter Android apps (see `scripts/mobile/build-all-apks.sh` `APPS` array):

- `poll-clerk` — election official
- `observer` — observer
- `party-agent` — party agent
- `field-support` — technician
- `personnel` — staff
- `citizen-verify` — public verification
- `bvr-kit` — biometric voter registration (**hardware-gated**: FAP60)
- `bvi-kit` — biometric verification identification (**hardware-gated**: AntIris)
- `diaspora-kit` — combined diaspora
- `legacy-android` — shim (WP-3.8)
- `rts-kit` — results transmission (GSM modem)
- `lms-offline` — offline LMS
- `accred-scanner` — accreditation scanner (CRL endpoint)
- `hello-flutter` — CI validator

Excluded (KMP/desktop, no Android target): `ro-desktop`, `kit-l1`.

## Flags reference

| Flag | Meaning |
|---|---|
| `--release` | Release APK |
| `--debug` | Debug APK (default) |
| `--analyze` | Run `flutter analyze` before build |
| `--test` | Run `flutter test` before build |
| `--clean` | `flutter clean` before build |
| `--no-pub-get` | Skip workspace `flutter pub get` |
| `--parallel` | Build 2 apps concurrently |
| `--app=<name>` | Build only one app |
| `--skip=a,b` | Skip named apps |
| `--keystore=...` | Java keystore path |
| `--keystore-pass=...` | Keystore password |
| `--key-alias=...` | Key alias |
| `--sign` | Enable signing (requires all 3 keystore flags) |
| `-h`, `--help` | Comprehensive usage |

## Hardware gates (M3)

`bvr-kit` and `bvi-kit` build fine but biometric capture requires physical USB devices:

- FAP60 fingerprint scanner (SDK: `FUFingerSDK.LIVESCAN_Init` / `captureFinger`)
- AntIris iris camera (SDK: `USBControl.antIrisIrexCapture` / `IrisIso.generate`)

Without these devices the drivers return `FingerprintResult.invalid()` / `IrisResult.invalid()` gracefully (not a crash). Skip them in CI with `--skip=bvr-kit,bvi-kit`.

See `docs/mobile/M3_HARDWARE_INTEGRATION.md` for SDK method signatures.

## Output

- APKs: `build/apks/<app>-<mode>.apk`
- Logs: `build/apks/logs/<app>.log`
- Build log: `build/apks/build.log`
- JSON summary: `build/apks/summary.json`

## Signing

```bash
# 1. Create keystore (once)
keytool -genkey -v -keystore keystore.jks \
  -alias eel-key -keyalg RSA -keysize 2048 -validity 10000 \
  -storepass <password> -keypass <password>

# 2. Create android/app/key.properties (do NOT commit)
#    storePassword=<password>
#    keyPassword=<password>
#    keyAlias=eel-key
#    storeFile=keystore.jks

# 3. Build signed release
scripts/mobile/build-all-apks.sh --release \
  --keystore=keystore.jks --keystore-pass=<password> \
  --key-alias=eel-key --sign
```

## CI usage

```bash
# Example Woodpecker step
- scripts/mobile/build-all-apks.sh \
    --release --analyze --test \
    --skip=bvr-kit,bvi-kit --parallel
```
