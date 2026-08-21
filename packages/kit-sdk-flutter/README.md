# kit-sdk-flutter

Secure offline-capable Flutter client SDK for eel kit applications (M01/M05/M08/M24/M25/M27). Backed by `verifier-rs` via Dart FFI.

## Purpose in context

Kit devices (BVD kits, biometric enrolment kits, RTS tallying kits, field support kits) share a common Flutter foundation. This package provides everything above the device HAL: secure local storage, offline queue, Ed25519/Merkle verifier FFI, HLC clock, sync engine, time-fence enforcement, duress detection, accessibility widgets, and SPKI-pinned HTTP client.

## Sections

### Isar / SQLCipher secure store

- `EelStore` - Isar database instance encrypted with SQLCipher; key derived from device attestation key
- `VoterRecordCache` - typed collection for offline voter lookup
- `OutboxQueue` - Isar collection of pending events to be synced
- Auto-purge on TTL expiry for ephemeral data

### Offline queue

- `OfflineQueueManager` - enqueues events when connectivity is absent; drains to the backend when online
- `ConnectivityWatcher` - Riverpod provider wrapping `connectivity_plus`
- At-least-once delivery with idempotency key attached to every queued event

### Attestation

- `DeviceAttestation` - Android Play Integrity / iOS DeviceCheck wrapper; result is a signed JWT included in every API request as `X-Eel-Attestation`
- `AttestationProvider` - Riverpod provider; refreshes attestation token before expiry

### HLC

- `EelHlcClock` - Dart port of `eel-hlc-rs`; wire-compatible `physical_ms.counter.node` format
- Used by `OfflineQueueManager` to timestamp queued events for causal ordering on sync

### Sync engine

- `SyncEngine` - bidirectional sync: pulls server delta since last HLC, pushes offline queue; conflict resolution is last-write-wins by HLC
- `SyncStatus` - Riverpod provider exposing `idle | syncing | conflict | error`

### Time-fence

- `TimeFenceGuard` - enforces election-day time windows; blocks prohibited operations outside the configured window
- `TimeFenceConfig` - loaded from backend on sync; stored in Isar for offline enforcement
- Duress mode: if the system clock is > 5 min ahead of the HLC received from server, raises `ClockManipulationSuspect`

### Duress

- `DuressDetector` - monitors for coercion indicators (silent alarm PIN, accelerometer pattern)
- `DuressMode` - when active, logs a silent alert event to the outbox without revealing to the operator

### Accessibility widgets

- `EelFormField` - WCAG 2.2 AA compliant input with Swahili/English labels
- `EelBiometricPrompt` - accessible biometric capture prompt with screen-reader support
- `EelStatusBanner` - high-contrast status banner for low-light field conditions
- All widgets pass colour contrast ratio >= 4.5:1 and support TalkBack/VoiceOver

### Verifier FFI

- `EelVerifier` - Dart bindings to `libeel-verifier.so` (built from `verifier-rs`)
- `EelVerifier.verifyInclusion(sth, proof, leafHash) -> VerifierResult`
- `EelVerifier.verifyConsistency(oldSth, newSth, proof) -> VerifierResult`
- `EelVerifier.verifySthSignature(sth) -> VerifierResult`

## Layout

```
kit-sdk-flutter/
  lib/
    eel_kit_sdk.dart          # main barrel export
    src/
      store/                  # Isar store
      queue/                  # offline queue
      attestation/            # device attestation
      hlc/                    # HLC clock
      sync/                   # sync engine
      time_fence/             # time fence
      duress/                 # duress detection
      widgets/                # accessibility widgets
      verifier/               # Dart FFI bindings
      http/                   # SPKI-pinned HTTP client (Dio)
  src/                        # native verifier shared library
  test/
  tests/
  pubspec.yaml
```

## Usage

```dart
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

// Startup
await EelStore.init(encryptionKey: attestation.derivedKey);
await SyncEngine.instance.sync();

// Offline queue
await OfflineQueueManager.instance.enqueue(Envelope(
  eventType: 'eel.rts.form34a_captured.v1',
  data: {'station_id': '47-001'},
));

// Verifier
final result = EelVerifier.verifyInclusion(sth, proof, leafHash);
if (result != VerifierResult.ok) throw VerificationException(result);
```

## Config

Loaded from `assets/config.json` in the consuming app; overridden by values received from the backend on sync.

| Key | Description |
|---|---|
| `sync_endpoint` | Backend sync URL |
| `attestation_endpoint` | Attestation verification URL |
| `time_fence_windows` | JSON array of allowed operation windows |

## Build

```bash
flutter pub get
flutter build apk --release   # Android
flutter build ios --release    # iOS
just build-verifier-ffi        # rebuild libeel-verifier.so before flutter build
```

## Test

```bash
flutter test
flutter test integration_test  # requires a connected device or emulator
```

## Related docs

- `packages/verifier-rs/README.md` - native verifier backing the FFI
- `packages/event-schemas/README.md` - event schemas for queued events
- `packages/i18n/README.md` - Swahili/English locale files
- `docs/adr/0017-kit-sdk.md` - architecture and offline-first rationale

## Limitations

The Dart FFI binding requires `libeel-verifier.so` to be built separately and placed in `src/` before the Flutter build. The `just build-verifier-ffi` command automates this. SQLCipher is bundled as a static library; the app binary is larger than a standard Flutter APK. Plan for 15-20 MB additional size.
