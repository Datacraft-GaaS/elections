# eel_design

Flutter consumption of `packages/design-system/tokens.json` (plan
`docs/mobile/plan.md` §5.2, Wave M0.1). WCAG 2.2 AA, colour-blind safe,
dark-mode ready, field-first ergonomics.

## Contents

- `EelTheme` — light + dark `ThemeData` with `EelColors` `ThemeExtension`
  (`context.eelColors.success`).
- `EelSpacing` / `EelRadius` / `EelSizing` / `EelMotion` / `EelTypography` /
  `EelFocus` — typed constants generated from tokens.
- Component library (§5.2 — apps must not hand-roll these):

| Widget | Purpose |
|---|---|
| `EelScaffold` | safe-area + banner slot + bottom action bar slot |
| `EelButton` | primary/secondary/destructive/ghost, loading, 48 dp |
| `EelTextField` / `EelPinField` | label always visible, error slot |
| `EelSelect` / `EelSearchField` | inputs matching the text field |
| `EelStatusChip` / `EelSlaTimerChip` / `EelSyncIndicator` | status display |
| `EelBanner` | info/warn/critical, accessible tone announcement |
| `EelConnectivityBanner` / `EelConnectivityChip` | §2.6 connectivity UX |
| `EelEmptyState` / `EelLoadingState` / `EelErrorState` / `EelOfflineState` / `EelPermissionRationale` | state matrix |
| `EelSecureScreen` | FLAG_SECURE / isCaptured wrapper |
| `EelStepFlow` | numbered wizard with resume |
| `EelResultCard` | signed-data display with proof status |
| `EelSealStamp` | seal verification display (ported) |
| `EelVoiceGuideButton` | TTS guidance toggle (ported) |
| `EelMinHitTarget` | ≥ 48 dp hit target wrapper |

## Regenerating tokens

```bash
cd packages/design-system/flutter
dart run tool/codegen_tokens.dart   # reads ../tokens.json
```

`test/token_parity_test.dart` fails if generated maps drift from
`tokens.json`. Committed generated files keep CI hermetic.

## Gates

- `flutter analyze` — 0 issues
- `flutter test` — unit + widget + goldens (`test/goldens/*.png`,
  regenerate with `flutter test --update-goldens`)
- Text scale 2.0 without layout overflow (golden test)