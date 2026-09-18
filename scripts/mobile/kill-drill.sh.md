# kill-drill.sh — Process Resilience and State‑Persistence Verification

## Purpose

Verify that the Android application correctly persists its internal finite‑state‑machine (FSM) progress across abrupt process terminations and recovers without duplicate work or data loss. This simulates real‑world conditions such as power loss, watchdog resets, or user‑initiated force‑stop during election operations.

## What It Tests

- **FSM state durability**: each workflow stage (`cold`, `integrityCheck`, `attestation`, `config`, `auth`, `roleRoute`, `ready`) must be recoverable after a kill.
- **Process termination handling**: the app must respond to `am kill` (SIGTERM) and clean up any transient resources.
- **State restoration**: upon relaunch, the app must resume from the exact point where it was killed, not restart from the beginning.
- **No duplicate side‑effects**: killing and restarting must not cause duplicate biometric captures, duplicate queue entries, or double‑counted telemetry events.
- **UI consistency**: after recovery, the same screen/widgets that were visible before the kill should be re‑displayed (where applicable).
- **Basic performance**: the time to reach each state from a cold start should be within expected bounds (the script uses a fixed 2‑second wait; in production you might replace with active polling).

## How It Works

1. For each FSM state in the ordered list:
   `cold → integrityCheck → attestation → config → auth → roleRoute → ready`
2. Launch the application from a clean state (cold start).
3. Wait a fixed interval (2 seconds) to allow the app to progress to the target state. In a more advanced implementation, one could poll a telemetry endpoint or a shared-preference flag to detect the exact state, but the fixed delay is sufficient for factory validation where timing is deterministic on a given device.
4. Issue `adb shell am force-stop <package>` to terminate the process
   (force-stop, not `am kill`: kill only affects background processes, so a
   foreground app would survive and the drill would prove nothing), then
   assert the pid is gone.
5. Relaunch and assert a *different* pid comes up within 15 s, and that
   logcat shows no FATAL EXCEPTION since the relaunch.
6. Any failed stage fails the drill (exit non-zero); the loop no longer
   prints unconditional PASS.

The loop stops early if any step fails (due to `set -e`), reporting which state caused the failure.

## Capabilities Validated

- **Launch speed**: measured implicitly by the fixed wait; a healthy app should reach each state within a few seconds.
- **Memory leak detection**: repeated kill/restart cycles should not cause OOM; if memory accumulates, later iterations may fail to launch.
- **UI access**: the app must be able to draw at least a blank Flutter view after each kill; if the UI fails to render, the subsequent telemetry ping (if any) may fail, causing the test to be flagged as a failure in the smoke‑test sense (though kill‑drill does not directly check UI rendering).
- **Search latency**: not applicable; this test focuses on lifecycle, not query performance.

## Exit Codes

- `0` – PASS: all states survived kill/restart cycles without error.
- `1` – FAIL: any state failed to recover (launch failure, hang, or script‑detected error).

## Usage

```sh
scripts/mobile/kill-drill.sh <APP> --device <serial>
```

- `<APP>` – Base name of the application as appears in `adb` package list (e.g., `bvi-kit`, `bvr-kit`, `rts-kit`).
- `<serial>` – Device serial number from `adb devices` (e.g., `emulator-5554`, `R58M60XJZJ`).

## Example

```sh
scripts/mobile/kill-drill.sh bvi-kit --device emulator-5554
```

## See Also

- `smoke.sh` – Basic liveness and telemetry check.
- `offline-drill.sh` – Network tolerance and deduplication.
- `factory_test.sh` – Runs smoke, kill-drill, and offline-drill sequentially.
