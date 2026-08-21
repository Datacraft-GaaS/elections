# legacy-android-shim/double

Canned-response mock for legacy Android device gRPC interface.

## Running the mock

```bash
python -m eel_double_legacy_android_shim --port 9091
```

Listens on localhost:9091. Accepts legacy payloads, returns CloudEvents.

## Chaos knobs

Environment variables for fault injection:

- ANDROID_DOUBLE_CHAOS: none | schema_error | duplicate | sig_mismatch (default: none)
  - schema_error: return malformed CloudEvent
  - duplicate: return same event twice on single call
  - sig_mismatch: alter payload after signing

- ANDROID_DOUBLE_DEVICE_ID: override device identity (default: random UUID)

- ANDROID_DOUBLE_FAIL_RATE: fraction of requests to fail (default 0.0)

## Canned payloads

Mock can simulate all four operation types (REG, AUTH, SYNC, VOTE). Golden test corpus in `contract-tests/fixtures/legacy_payloads.json`.

## Device cert simulation

Double does not validate device certs. All requests accepted. Shim re-signature is synthetic (dev only).
