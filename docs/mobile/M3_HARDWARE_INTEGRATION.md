# M3 Hardware Integration

**Status:** Structural implementation complete. Real SDK APIs integrated. Feature activation requires hardware procurement.

**Wave:** M3 (Kits — biometric-edge-svc, rts-svc)  
**Hardware gate:** FAP60 fingerprint scanner + AntIris iris camera + HSM module + GSM modem  
**Plan.md:** §3 (M3 Kits), §11.0 (sequential enforcement)

---

## Overview

M3 replaces all `UnimplementedError` stub-throws with real platform-channel integrations backed by vendor SDKs. Two SDK packages were analysed and integrated:

| Device | SDK Path | Vendor | Interface |
|--------|----------|--------|-----------|
| FAP60 fingerprint scanner | `sdks/2.fap60-android/` | FingerUp (ZAZ) | USB HID + UVCCamera |
| AntIris iris camera | `sdks/AntIris_Android_SDK/` | Antiris | USB UVC + custom IrisIso |

The remaining two devices (HSM key module, GSM modem) have no SDK on disk yet. Drivers for those return structured error results rather than throwing.

---

## 1 — FAP60 Fingerprint Scanner (M01 / bvr-kit)

### SDK artefacts

```
sdks/2.fap60-android/
├── FunctiontestDemo_Android_api351202/
│   ├── libUvcFinger/src/main/java/
│   │   ├── cn/fingerup/
│   │   │   ├── FUFingerSDK.java      ← USB scanner host (UVC camera + HID)
│   │   │   └── FingerAlgo.java       ← Distortion correction + flat/roll finger
│   │   └── com/ZHIANG/
│   │       ├── FPAPI.java            ← ANSI/ISO template algorithm (WSQ, match)
│   │       └── HostUsb.java          ← USB device authorisation
│   └── libUvcFinger/libs/
│       ├── arm64-v8a/               ← Native .so files (FAP60 hardware)
│       │   ├── libFingerILA.so
│       │   ├── libFpStdAPI.so
│       │   ├── libZAZ_FpStdAPI.so
│       │   ├── libJniFinger.so
│       │   ├── libfingeralgo.so
│       │   ├── libUVCCamera.so      ← Serenegiant UVC driver
│       │   ├── libusb100.so
│       │   └── libjpeg-turbo1500.so
│       └── armeabi-v7a/             ← 32-bit fallback
```

### Key API surface

#### FUFingerSDK (USB scanner)

| Method | Returns | Notes |
|--------|---------|-------|
| `LIVESCAN_Init(isLcdScreenMode: bool)` | `int` (0=success) | Detects USB VID=0x0507 PID=0x0581 |
| `LIVESCAN_Close()` | `int` | Releases USB device |
| `captureFinger(timeoutMs, rawBuffer, bmpBuffer)` | `int` | Blocks up to timeout. Returns 0 on success. |
| `LIVESCAN_GetFPRawData(channel, buffer)` | `int` | Raw 1600×1500 grayscale |
| `LedLight(index) / LcdImage(index)` | void | UI feedback |
| `LIVESCAN_GetVersion()` | `int` (=100) | Self-check |
| `ERR_BUSY = -5` | | Device in use |
| `ERR_DEVICE_NOT_FOUND = -4` | | No FAP60 detected |
| `ERR_TIMEOUT = -10` | | Finger not detected in time |

#### FPAPI (Template algorithm)

| Method | Returns | Notes |
|--------|---------|-------|
| `F_OpenDevice()` | `long` handle (>0=success) | Opens FPAlgorithm VID=0x28E9 PID=0x2018 |
| `F_CloseDevice(handle)` | `int` | |
| `F_GetImage(handle, image[256×360])` | `int` | Capture one flat finger |
| `F_GetImageQuality(handle, image)` | `int` (0–100) | NIST quality score |
| `F_CreateTemplate(handle, image, template[1024])` | `int` | ANSI template; returns size |
| `F_CompareTemplates(handle, t1, t2)` | `int` (0–100) | Match score |
| `F_SearchingISOTemplates(handle, query, count, db[], threshold)` | `int` index or -1 | 1:N search |
| `F_CompressToWSQImage(handle, raw, w, h, wsqOut)` | `int` | WSQ compression |
| `DEF_QUALITY_SCORE = 50` | | Minimum acceptable quality |
| `DEF_MATCH_SCORE = 45` | | Minimum match threshold |

### Integration files written

| File | Purpose |
|------|---------|
| `services/biometric-edge-svc/android/app/src/main/java/com/eel/biometric/Fap60Plugin.kt` | Flutter Android plugin — registers `MethodChannel('eel.bvr/fingerprint')`, wraps FUFingerSDK + FPAPI calls, returns structured `Map` results |
| `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/drivers/fap60/fap60_driver.dart` | Dart driver — calls plugin, returns `Fap60Result` with `templateBase64`, `qualityScore`, `isValid` |
| `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/drivers/bvr_driver.dart` | Updated — `captureFingerprint()` delegates to `Fap60Driver`; gracefully returns error result when plugin not registered |

### Registering the plugin (Android)

In `biometric-edge-svc/android/app/src/main/kotlin/.../MainActivity.kt` or `Application.kt`:

```kotlin
import com.eel.biometric.Fap60Plugin

// In onCreate:
Fap60Plugin().register(
    flutterEngine.pluginRegistry.registrarFor("com.eel.biometric.Fap60Plugin")!!,
    flutterEngine.dartExecutor.binaryMessenger
)
```

Or via `GeneratedPluginRegistrant` (auto-registered by `flutter pub get`):

```kotlin
// In GeneratedPluginRegistrant.kt (after adding to pubspec.yaml):
import com.eel.biometric.Fap60Plugin
Fap60Plugin.registerWith(registry.registrarFor("com.eel.biometric.Fap60Plugin")!!)
```

---

## 2 — AntIris Iris Camera (M05 / bvi-kit)

### SDK artefacts

```
sdks/AntIris_Android_SDK/
├── libs/
│   ├── antiris.antusbcamera.jar       ← USBControl + callbacks
│   ├── serenegiant.jar                ← UVCCamera base
│   └── jniLibs/
│       ├── arm64-v8a/
│       │   ├── lib_Antiris.so         ← Iris algorithm
│       │   └── libUSBCamera.so        ← UVC camera driver
│       └── armeabi-v7a/
├── Sample/AndroidStudio/IrisTest/src/main/java/com/antiris/
│   ├── MainActivity.java               ← Reference app
│   └── IrexCaptureDialog.java         ← Username capture dialog
└── Document/AntIris_Android_SDK_en.pdf  ← API reference
```

### Key API surface (USBControl / AntUSBCamera)

| Method | Returns | Notes |
|--------|---------|-------|
| `USBControl.getInstance()` | `USBControl` | Singleton |
| `init(activity, textureView, DeviceConnectCallback, boolean)` | void | Opens UVC camera |
| `isCameraOpened()` | `boolean` | |
| `isIrexCapturing()` | `boolean` | |
| `antIrisIrexCapture(user, eyeIndex, irexType, flag, timeoutMs, IrexCaptureCallback)` | void | K1/K2/K3/K7 capture |
| `antStopIrexCapture(boolean)` | void | Abort capture |
| `getK7Image(eye)` | `byte[]` | K7-format iris image |
| `getK7ImgWidth(eye) / getK7ImgHeight(eye)` | `int` | |
| `getk7Quality(eye)` | `int` | Quality 0–100 |
| `grayToJp2(data, w, h, jp2Out, jp2Size, irexType, mode)` | void | Compress to JPEG2000 |
| `getBackgroundBytes(flip)` | `byte[]` | Background subtraction |
| `saveGrayToPng(data, path, name, w, h)` | void | Debug output |
| `IRLED_OnOff(boolean)` | void | IR illumination control |
| `setStreamSpeedLevel(level)` | void | HIGH=0 / FULL=1 / LOW=2 |

**IrexType:** K1=1, K2=2, K3=3, K7=7 (multi-iris capture modes)  
**EyeIndex:** 0=left, 1=right  
**IrexCaptureCallback:** `onSuccess(eye, user, temp)`, `onError(msg)`, `onProcess(eye, progress, distance, opening)`

### Key API surface (IrisIso)

```java
IrisIso.generate(
    int version,           // 0
    byte[] jp2Data,        // compressed iris image
    int width, int height, // K7 dimensions
    byte[] serialBytes,     // camera serial number (UTF-8)
    int quality,            // quality score from getk7Quality()
    int irexType,          // K1/K2/K3/K7
    int format,             // ANT_IMAGE_FORMAT_JPEG2000
    int compressionMode,     // ANT_COMPRESSION_MODE_LOSSY
    int compressionRatio     // 8
) → byte[]                 // ISO 19794-6 iris binary
```

### Integration files written

| File | Purpose |
|------|---------|
| `services/biometric-edge-svc/android/app/src/main/java/com/eel/biometric/AntIrisPlugin.kt` | Flutter Android plugin — registers `MethodChannel('eel.bvi/iris')`, implements full capture lifecycle (init → capture → ISO generation → release) |
| `services/biometric-edge-svc/apps/bvi-kit/lib/src/core/drivers/antiris/antiris_driver.dart` | Dart driver — calls plugin, returns `AntIrisResult` with `imageBase64`, `isoBase64`, `quality`, `width`, `height` |
| `services/biometric-edge-svc/apps/bvi-kit/lib/src/core/drivers/bvi_driver.dart` | Updated — `captureIris()` delegates to `AntIrisDriver`; `fixLocation()` stubs gracefully (geolocator required) |

---

## 3 — Driver Architecture

```
bvr-kit / lib / src / core / drivers /
├── bvr_driver.dart              ← Public interface (MrzScanResult, FingerprintResult, IprsVerifyResult)
└── fap60 /
    └── fap60_driver.dart       ← Fap60Driver → MethodChannel → Fap60Plugin (Kotlin)

bvi-kit / lib / src / core / drivers /
├── bvi_driver.dart              ← Public interface (NtsCaptureResult, GnssFixResult, AntIrisResult)
└── antiris /
    └── antiris_driver.dart     ← AntIrisDriver → MethodChannel → AntIrisPlugin (Kotlin)
```

### BvrDriver (bvr-kit, M01)

```dart
class BvrDriver {
  // Real: FAP60 SDK via MethodChannel
  Future<FingerprintResult> captureFingerprint(String position);
  // Returns error result (not exception) when plugin not registered
}
```

### BviDriver (bvi-kit, M05)

```dart
class BviDriver {
  // Real: AntIris SDK via MethodChannel
  Future<AntIrisResult> captureIris({String userName, int eyeIndex});
  // Network call: NTS service (no SDK, endpoint required)
  Future<NtsCaptureResult> captureNts();
  // Platform: geolocator package (no SDK, package required)
  Future<GnssFixResult> fixLocation();
}
```

---

## 4 — Still Hardware-Gated

These two device categories have no SDK on disk. Drivers return structured error results:

### HSM Key Module (M24 / diaspora-kit)

**Driver:** `services/biometric-edge-svc/apps/diaspora-kit/lib/src/core/drivers/diaspora_driver.dart`  
**SDK:** Not yet on disk  
**Driver state:** Returns `HsmKeyResult.invalid('HSM hardware required (plan.md M3)')`  
**Integration path:** PKCS#11 / vendor SDK → `DiasporaDriver.generateRealKey()`

### GSM Modem / SMS Fallback (M08 / rts-kit)

**Driver:** `services/rts-svc/apps/rts-kit/lib/src/core/drivers/rts_driver.dart`  
**SDK:** Not yet on disk  
**Driver state:** Returns `SmsStatus.invalid('GSM modem required (plan.md M3)')`  
**Integration path:** AT command interface / telco SDK → `RtsDriver.sendSmsFallback()`

---

## 5 — M3 Session Bootstrap

Per plan.md M3: all kits run a bootstrap FSM before feature screens open.

```
cold → capabilityCheck → selfCheck → roleRoute → ready / blocked
```

**Files:**
- `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/session/m3_session.dart` — `M3BootstrapService`
- `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/capability/capability_status.dart` — `KitCapabilityStatus`
- `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/capability/m3_feature_gate.dart` — `M3FeatureGate` widget
- `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/platform_channel/bvr_channels.dart` — channel namespace constants

**Feature gate behaviour:**

| Capability state | Bootstrap step | Feature screen |
|----------------|----------------|----------------|
| All hardware present | `ready` | Open |
| Any hardware missing | `blocked` | `M3FeatureGate` banner: "Hardware required: [list]" |
| Self-check failure | `error` | `M3FeatureGate` banner: "[device] self-check failed" |

---

## 6 — Test Harness (CI-safe)

Hardware-mocked tests run without devices:

```dart
// services/biometric-edge-svc/apps/bvr-kit/lib/src/core/drivers/platform/m3_integration_test.dart

class MockBvrDriver implements BvrDriver {
  @override Future<FingerprintResult> captureFingerprint(String position) async =>
      const FingerprintResult(templateBase64: 'A1B2C3D4=', qualityScore: 92,
        fingerPosition: 'LEFT_INDEX', isValid: true);
}
```

CI passes; feature-screen logic verified against mock results.

---

## 7 — Checklist: Activating M3 Features

When a FAP60 or AntIris device is physically connected:

### FAP60 (bvr-kit)

- [ ] Copy `libUvcFinger/libs/` `.so` files into `android/app/src/main/jniLibs/`
- [ ] Add `antiris.antusbcamera.jar` and `serenegiant.jar` to `android/app/libs/`
- [ ] Add Fap60Plugin to `GeneratedPluginRegistrant`
- [ ] `BvrDriver.selfCheck()` returns `true`
- [ ] `M3SessionState.step` → `M3BootstrapStep.ready`
- [ ] Feature gate opens; biometric capture UI becomes functional

### AntIris (bvi-kit)

- [ ] Copy `jniLibs/` `.so` files into `android/app/src/main/jniLibs/`
- [ ] Add `antiris.antusbcamera.jar` to `android/app/libs/`
- [ ] Add AntIrisPlugin to `GeneratedPluginRegistrant`
- [ ] Grant USB permission in `AndroidManifest.xml` (`<uses-feature android:name="android.hardware.usb.host"/>`)
- [ ] `BviDriver.selfCheck()` returns `true`
- [ ] `M3SessionState.step` → `M3BootstrapStep.ready`
- [ ] Iris capture UI becomes functional

---

## 8 — Files Changed / Created

```
services/biometric-edge-svc/
├── android/app/src/main/java/com/eel/biometric/
│   ├── Fap60Plugin.kt          [NEW] FAP60 platform channel
│   └── AntIrisPlugin.kt        [NEW] AntIris platform channel
└── apps/bvr-kit/lib/src/core/
    ├── drivers/
    │   ├── bvr_driver.dart     [UPDATED] delegates to Fap60Driver
    │   └── fap60/
    │       └── fap60_driver.dart  [NEW] Dart SDK wrapper
    ├── session/
    │   └── m3_session.dart    [NEW] M3 bootstrap FSM
    ├── capability/
    │   ├── capability_status.dart   [NEW] hardware probe
    │   └── m3_feature_gate.dart     [NEW] feature screen gate
    ├── platform_channel/
    │   └── bvr_channels.dart   [NEW] channel constants
    └── drivers/platform/
        └── m3_integration_test.dart  [NEW] CI-safe mock harness

services/biometric-edge-svc/apps/bvi-kit/lib/src/core/
├── drivers/
│   ├── bvi_driver.dart         [UPDATED] delegates to AntIrisDriver
│   └── antiris/
│       └── antiris_driver.dart    [NEW] Dart SDK wrapper
```

---

## 9 — SDK Source References

| SDK | Root | Primary class | Key method |
|-----|------|-------------|------------|
| FAP60 Fingerprint | `sdks/2.fap60-android/FunctiontestDemo_Android_api351202/libUvcFinger/src/main/java/` | `FUFingerSDK` | `captureFinger()` |
| FAP60 Algorithm | `com/ZHIANG/FPAPI.java` | `FPAPI` | `F_CreateTemplate()`, `F_CompareTemplates()` |
| FAP60 HID | `cn/fingerup/HidDevice.java` | `HidDevice` | `Init_Device()` |
| AntIris Camera | `libs/antiris.antusbcamera.jar` → `com.antiris.AntUSBCamera.USBControl` | `USBControl` | `antIrisIrexCapture()` |
| AntIris ISO | `com.antiris.AntUSBCamera.iso.IrisIso` | `IrisIso` | `generate()` |

---

*Plan.md M3 status: structural implementation complete. Hardware procurement (FAP60 + AntIris + HSM + GSM) is the only remaining gate. All stub-throws replaced with real SDK calls or graceful error-result returns.*

---

## 10 — GSM Modem / SMS Fallback (M08 / rts-kit)

### SDK / API status

**No vendor SDK required.** GSM modems expose the universal **3GPP AT command set** (`AT+CMGF`, `AT+CMGS`, `AT+CMGL`, `AT+CPMS`) through USB serial (`/dev/ttyUSB0` or `/dev/ttyACM0`).

### Integration files

| File | Purpose |
|------|---------|
| `services/rts-svc/apps/rts-kit/lib/src/core/at_command/at_command.dart` | AT command vocabulary (`AtCommand`, `AtResponse`, `AtSerialTransport` abstract class) |
| `services/rts-svc/apps/rts-kit/lib/src/core/at_command/usb_serial_transport.dart` | Real transport using `usb_serial` package (USB device discovery + read/write) |
| `services/rts-svc/apps/rts-kit/lib/src/core/at_command/gsm_modem.dart` | `GsmModem` class: init → send text SMS (`AT+CMGS`) / PDU SMS (`encodeSmsSubmit`) / read SMS / delete SMS |
| `services/rts-svc/apps/rts-kit/lib/src/core/sms_pdu/sms_pdu.dart` | SMS-SUBMIT PDU encoder (GSM 7-bit / 8-bit binary, multi-segment UDH) |
| `services/rts-svc/apps/rts-kit/pubspec.yaml` | Added `usb_serial: ^0.5.0` |
| `docs/mobile/M3_SMS_USB_MANIFEST.md` | AndroidManifest.xml USB host permission snippet + VID/PID filter |

### Usage (Dart)

```dart
final modem = GsmModem();
final ok = await modem.init(portPath: '/dev/ttyUSB0', baud: 115200);
if (!ok) return; // No modem detected
final sent = await modem.sendTextSms(
  destination: '+254700123456',
  text: 'RTS:OK',
);
final received = await modem.readReceivedSms();

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
