# offline-drill.sh — Network Tolerance and Deduplication Validation

## Purpose

Verify that the Android application correctly queues operations while offline (no network connectivity) and, upon reconnection, transmits the queued records exactly once without duplication or loss. This simulates real‑world scenarios such as entering a basement, tent, or area with intermittent cellular coverage during election work.

## What It Tests

- **Airplane mode handling**: the app must detect loss of connectivity and gracefully degrade to an offline queue mode.
- **Offline queuing**: biometric captures, form submissions, telemetry events, and other write‑operations must be persisted locally (e.g., via SQLite, shared preferences, or a secure store) when the network is unavailable.
- **Network recovery**: upon restoration of connectivity, the app must automatically resume transmission of queued records.
- **Deduplication**: each logical operation must be transmitted exactly once, even if the offline‑to‑online transition occurs multiple times or if the app is restarted while offline.
- **Idempotency safeguards**: the backend must accept duplicate submissions safely, but the client should avoid sending them in the first place to conserve bandwidth and battery.
- **UI responsiveness**: the application should remain usable while offline, allowing the operator to continue capturing data without waiting for network timeout.

## How It Works

1. Enable airplane mode and **verify** `settings get global airplane_mode_on`
   reports 1 (retries once; fails otherwise).
2. Wait briefly to allow the radios to power down and the app to observe the
   connectivity loss.
3. Launch the app and assert it stays alive and crash-free (no FATAL
   EXCEPTION) while offline — an offline crash fails the drill. Driving the
   app through a full workflow to queue records is still future work (use
   `adb shell input` events or the integration-test driver).
4. Disable airplane mode, **verify** the setting reports 0, wait 5 s, assert
   the app process survived the transition.
5. Record-level dedup: when `EEL_DEDUP_CHECK` is set, it is executed (exit 0
   required — e.g. a command querying the BFF audit log). When unset, dedup
   is reported SKIP explicitly, never as a silent pass.

The script currently focuses on the mechanics of toggling airplane mode and provides clear comments where future automation (integration test driving, backend verification) should be inserted. Even in its current form, it validates that the device can enter and exit airplane mode without crashing the app, which is a prerequisite for the offline drill.

## Capabilities Validated

- **Radio state switching**: the OS must correctly honor airplane mode toggles and report connectivity changes to the app.
- **App resilience**: the application must not crash when the underlying network interfaces disappear or reappear.
- **Basic timing**: the delays (2 s offline, 5 s online) are conservative averages; on faster hardware the app may transition sooner.
- **Memory usage**: while not measured, the offline queue must be able to hold at least the records generated during the offline window without exhausting memory.
- **UI access**: the app should remain in the foreground and responsive while offline; if it crashes or becomes unresponsive, subsequent steps would fail.
- **Search latency**: not applicable; this test concerns network layering, not query performance.

## Exit Codes

- `0` – PASS: airplane mode toggled successfully and the script reached the end (the TODO sections are not considered failures).
- `1` – FAIL: missing device parameters, failure to toggle airplane mode, or any unexpected error (due to `set -e`).

## Usage

```sh
scripts/mobile/offline-drill.sh <APP> --device <serial>
```

- `<APP>` – Base name of the application as appears in `adb` package list (e.g., `bvi-kit`, `bvr-kit`, `rts-kit`).
- `<serial>` – Device serial number from `adb devices` (e.g., `emulator-5554`, `R58M60XJZJ`).

## Example

```sh
scripts/mobile/offline-drill.sh bvi-kit --device emulator-5554
```

## See Also

- `smoke.sh` – Basic liveness and telemetry check.
- `kill-drill.sh` – Process resilience and state persistence.
- `factory_test.sh` – Runs smoke, kill-drill, and offline-drill sequentially.
