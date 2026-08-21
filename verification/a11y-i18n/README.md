# Accessibility and i18n sweep (WP-6.6)

Wave 6 verification of WCAG 2.2 conformance and internationalisation completeness
across every eel UI surface.

## Scope

| Criterion | Target |
|---|---|
| WCAG conformance (all surfaces) | 2.2 AA |
| WCAG conformance (M68 referendum-explainer) | 2.2 AAA |
| Locales | 11 production locales + xxa pseudo-locale |
| Screen readers | NVDA (Windows), TalkBack (Android), VoiceOver (macOS) |
| Frameworks covered | Astro, Phoenix LiveView, Django, Flutter, Tauri 2 |

## Files

| File | Purpose |
|---|---|
| `ui-inventory.yaml` | Canonical list of every UI surface with metadata |
| `axe-sweep.sh` | Runs axe-core via `tools/wcag-audit` against every URL; aggregates to `axe-report.json` |
| `pseudo-locale-check.sh` | Builds each UI with pseudo-locale `xxa`; asserts no string truncation via headless snapshot |
| `locales/11-locale-audit.yaml` | Per-locale x per-surface: catalogue completeness, font fallback, ICU plurals, RTL note |
| `screen-reader/nvda-scripts/` | Python NVDA scripts for portal and verifier web apps |
| `screen-reader/talkback-scripts/` | ADB-driven TalkBack traversal for all Android kits and mobile apps |
| `screen-reader/voiceover-scripts/` | AppleScript VoiceOver traversal for ro-desktop and rla-desktop (Tauri) |
| `report-template.md.tmpl` | Per-surface report template |
| `fixtures/local-a11y-report.md` | Worked example report |
| `tests/test_scripts.sh` | Smoke tests: bash -n clean, YAML valid, UI directories exist |

## Running the sweep

```
# Full sweep (requires browser + axe-core installed)
bash verification/a11y-i18n/axe-sweep.sh

# Pseudo-locale build check
bash verification/a11y-i18n/pseudo-locale-check.sh

# Validate scripts and YAML
bash verification/a11y-i18n/tests/test_scripts.sh
```

## 11 production locales

en-KE, sw-KE, om-ET, so-KE, kam-KE, mer-KE, guz-KE, luo-KE, luy-KE, kln-KE, mas-KE

None of these use RTL script. The RTL mechanism (CSS `dir="rtl"` + `bidi-override`, ICU BIDI) is documented
and exercised via the `xxa` pseudo-locale which includes directionality markers.

## WCAG target rationale

M68 (referendum-explainer) targets AAA because it is the primary public-information surface
for a constitutional referendum; citizens with low vision or cognitive disabilities must be
able to access referendum information without assistance.

## Screen-reader coverage

| Platform | Tool | Surfaces |
|---|---|---|
| Windows | NVDA 2024.x via `nvda_at_api` | public-portal, bvv-voter-lookup |
| Android | ADB + TalkBack | bvr-kit, rts-kit, party-agent, poll-clerk, observer-app, field-support, personnel, citizen-verify, diaspora-kit |
| macOS | VoiceOver via AppleScript | ro-desktop (Tauri), rla-desktop (Tauri) |
