# smoke.sh — Application Health Check

## Purpose

Verify that the Android application can be launched from a cold start, initializes its Flutter engine, binds to required platform services, and establishes a telemetry session with the Eel backend within a reasonable time window. This is the first line of defense in factory testing: if the app fails to start or telemetry does not come up, deeper testing is pointless.

## What It Tests

- Application packaging and installation integrity
- Ability to resolve the main activity from package metadata
- Successful cold launch via `am start`
- Initialization of Flutter engine and Dart isolates
- Binding to native plugins (e.g., telemetry, shared_preferences, etc.)
- Optional telemetry session probe against `EEL_TELEMETRY_URL` when configured
  (when unset, telemetry is reported SKIP — never faked)
- Crash-freedom: no FATAL EXCEPTION in logcat after launch
- Basic responsiveness: the app must report health within 30 seconds.

## How It Works

1. Validates that the specified `<APP>` fragment matches exactly one installed
   package (fails on zero or ambiguous matches).
2. Resolves the launcher activity via `cmd package resolve-activity --brief`
   (falls back to `monkey` with a warning when unresolvable).
3. Cold-launches with `am start -W`, records TotalTime, WARNs when over the
   2000 ms budget (plan §7).
4. Waits up to 30 s for a live process (`pidof`); fails when none appears.
5. Fails on any FATAL EXCEPTION in logcat since launch.
6. When `EEL_TELEMETRY_URL` is set, polls it for up to 30 s and fails on
   timeout; when unset, reports telemetry SKIP.

## Note on Telemetry Endpoint

Set `EEL_TELEMETRY_URL` to the session/health endpoint to probe (in typical
factory setups the host runs `adb reverse tcp:9090 tcp:9090` to forward the
device's localhost to the host's telemetry listener, then
`EEL_TELEMETRY_URL=http://localhost:9090/healthz`). When unset, the telemetry
step is reported SKIP — the script never pretends to have probed it.

## Capabilities Validated

- **Launch speed**: time from `am start` to first telemetry pulse (should be <5 s on decent hardware; the script allows up to 30 s to accommodate slow CI agents).
- **Memory sanity**: while not directly measuring RAM, a successful launch indicates the Dalvik/ART VM was able to allocate the initial heap.
- **UI access**: the launcher activity must be able to render at least a blank Flutter view; if the app crashes before first frame, telemetry will not initialize and the test fails.
- **Search latency**: not applicable; this is a liveness probe, not a feature‑specific test.

## Exit Codes

- `0` – PASS: telemetry session detected within timeout.
- `1` – FAIL: app not installed, launch failed, or no telemetry within timeout.

## Usage

```sh
scripts/mobile/smoke.sh <APP> --device <serial>
```

- `<APP>` – Base name of the application as appears in `adb` package list (e.g., `bvi-kit`, `bvr-kit`, `rts-kit`).
- `<serial>` – Device serial number from `adb devices` (e.g., `emulator-5554`, `R58M60XJZJ`).

## Example

```sh
scripts/mobile/smoke.sh bvi-kit --device emulator-5554
```

## See Also

- `kill-drill.sh` – Process resilience and state persistence.
- `offline-drill.sh` – Network tolerance and deduplication.
- `factory_test.sh` – Runs smoke, kill-drill, and offline-drill sequentially.
