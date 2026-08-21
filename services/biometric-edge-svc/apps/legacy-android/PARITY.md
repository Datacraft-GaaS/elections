# Legacy Android Parity Matrix

This document maps each BVI/BVR Kit feature to its availability on the legacy-android shim (WP-3.8).

| Feature | bvr-kit | bvi-kit | legacy-android | Notes |
|---|---|---|---|---|
| Fingerprint capture | Yes | Yes | Yes (software) | Quality gate thresholds remain the same |
| Iris capture | Yes | Yes | No | Iris sensor not available on legacy devices |
| Face capture | Yes | Yes | Yes | Camera-based; lower quality floor |
| MRZ scan | Yes | No | Yes | Uses device camera |
| TPM attestation | No | No | No | legacy-android has no TPM |
| Play Integrity attestation | No | No | Yes | Android 8+ Play Integrity API |
| App Attest attestation | No | No | No | iOS only |
| Software attestation | No | No | Yes | Marks attestation_tier=legacy on backend |
| GNSS spoof detection | No | Yes | Partial | Single-constellation only on legacy |
| IMU spoof detection | No | Yes | No | No IMU on most legacy devices |
| Time-fence FSM (EC-TIM-01) | No | Yes | Yes | NTP fallback used instead of NTS |
| Offline 7-day queue | No | Yes | Yes | |
| Distress finger (EC-IDN-08) | Yes | No | Yes | |
| Dual-officer close (EC-OPS-04) | No | Yes | Yes | |
| s.44A dual-officer enrolment | Yes | No | Yes | |
| Amputee/assisted path (EC-HUM-02) | Yes | No | Yes | |
| 11-language UI | Yes | Yes | Yes | |
| Accessibility (a11y) | Yes | Yes | Yes | |
| OTA update | Yes | Yes | Yes | Slower channel; no delta OTA |
| Boot time target <= 90 s | No | Yes | No | Legacy devices boot in 120-180 s |
| Enrolment throughput >= 12/hr | Yes | N/A | Partial | ~8/hr on reference legacy device |

## Deviation Rationale

- **No iris sensor**: Legacy Android devices (pre-2020) lack secure iris scanners. The s.44A fallback path applies.
- **No TPM**: Software attestation tier is used; backend audit flags all legacy-attested sessions.
- **Reduced GNSS**: Single-constellation GPS only; no multi-constellation cross-check. Risk accepted with physical security perimeter controls at polling stations.
- **No NTS**: NTP is used for time fence; drift tolerance extended to 10 min for legacy devices in poor-connectivity areas (deviation from EC-TIM-01; documented as EC-TIM-01-LEG).
