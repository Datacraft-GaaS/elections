<!-- M08 SMS Fallback — USB Serial + GSM Modem (plan.md M3)
     Add to android/app/src/main/AndroidManifest.xml -->

<!-- USB Host feature declaration -->
<uses-feature android:name="android.hardware.usb.host" android:required="true" />

<!-- USB device filter: GSM modem VID/PID patterns -->
<activity android:name=".MainActivity" ... >
  <intent-filter>
    <action android:name="android.hardware.usb.action.USB_DEVICE_ATTACHED" />
  </intent-filter>
  <meta-data
      android:name="android.hardware.usb.action.USB_DEVICE_ATTACHED"
      android:resource="@xml/usb_device_filter" />
</activity>

<!-- usb_device_filter.xml (res/xml/usb_device_filter.xml): -->
<!-- 
<resources>
  <usb-device vendor-id="0x12D1" product-id="0x14DB" />  <!-- Huawei -->
  <usb-device vendor-id="0x05C6" product-id="0x9001" />  <!-- Quectel EC25 -->
  <usb-device vendor-id="0x05C6" product-id="0x9009" />  <!-- SIMCom A767 -->
  <usb-device vendor-id="0x2FE5" product-id="0x3002" />  <!-- Telit -->
  <usb-device vendor-id="0x6547" product-id="0x0232" />  <!-- Sierra Wireless -->
  <usb-device vendor-id="0x1546" product-id="0x00A6" />  <!-- u-blox -->
</resources>
-->
---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
