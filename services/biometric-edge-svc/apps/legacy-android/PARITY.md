# Legacy Android Parity Matrix

This document maps each BVI/BVR Kit feature to its availability on the legacy-android shim (WP-3.8).
**Shim is intentional** (per docs/mobile/legacy-android.md): no parallel feature implementation.
Buttons deep-link into real kit apps (`eel://kit/bvr/enrol`, `eel://kit/bvi/identify`).
If kit apps are absent, MDM managed-app store install guidance is shown.

| Feature | bvr-kit | bvi-kit | legacy-android (shim) | Notes |
|---|---|---|---|---|
| Local fingerprint capture | Yes | Yes | No (deep-link) | Use bvr-kit |
| Local iris capture | Yes | Yes | No (deep-link) | Use bvi-kit |
| Local face capture | Yes | Yes | No (deep-link) | Use bvr-kit |
| Local MRZ scan | Yes | No | No (deep-link) | Use bvr-kit |
| TPM attestation | No | No | No | legacy-android has no TPM |
| Play Integrity attestation | No | No | Yes | Android 8+ Play Integrity API |
| App Attest attestation | No | No | No | iOS only |
| Software attestation (shim banner) | No | No | Yes | Marks attestation_tier=legacy on backend |
| s.44A enrolment path | Yes | No | No (deep-link) | Use bvr-kit |
| Local enrolment form | Yes | No | No (deep-link) | Use bvr-kit |
| 11-language UI | Yes | Yes | Yes | |
| Accessibility (a11y) | Yes | Yes | Yes | |
| OTA update | Yes | Yes | Yes | Slower channel; no delta OTA |
| Deep-link into bvr-kit | n/a | n/a | Yes | `eel://kit/bvr/enrol` |
| Deep-link into bvi-kit | n/a | n/a | Yes | `eel://kit/bvi/identify` |
| MDM managed-app install guidance | n/a | n/a | Yes | Shown when kit app not installed |

## Shim Contract

The shim is a gateway, not a feature app. Buttons in the home screen
perform one of two actions in order:

1. **Attempt deep-link** via `launchUrl('eel://kit/{bvr,bvi}/...', mode: externalApplication)`.
2. **On failure** (`launched == false` or platform exception), show an
   `AlertDialog` titled "Install Kit App" with text "Install {package} from
   MDM managed-app store to use hardware features." and two actions:
   - Cancel — dismiss dialog
   - Open Store — `launchUrl('https://eel.local/mdm/apps/{package}')`

`url_launcher` is the only new dependency. The shim retains software-only
attestation honesty (banner remains; backend marks `attestation_tier=legacy`).

## Definition of Done (per docs/mobile/legacy-android.md)

- [x] No dead controls (no `onPressed: () {}`)
- [x] Buttons deep-link to bvr-kit/bvi-kit (`eel://kit/...`)
- [x] Install guidance fallback when kit app absent
- [x] Attestation banner stays (software-only honesty)
- [x] PARITY.md updated (no parallel feature claims)
- [ ] Widget test: button paths + deep-link fallback
- [ ] Integration test (device): deep-link lands in target kit screen

## Notes

- **No iris on legacy devices**: Shim does not attempt iris capture. Users are deep-linked to bvi-kit on capable devices.
- **No TPM**: Backend marks `attestation_tier=legacy`; all such sessions flagged in audit.
- **No parallel features**: Per `docs/mobile/legacy-android.md`, the shim must not implement enrolment/identification logic — it only routes users to real kit apps.
