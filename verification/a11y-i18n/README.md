<title>Accessibility and i18n Verification</title>

# Accessibility and i18n Verification

WCAG 2.2 Level AA compliance check and internationalisation completeness verification for all
user-facing surfaces: portal, kit Flutter app, field-mobile app, and Django admin surfaces.

## Purpose

WP-6.6. Confirms that the platform meets Kenya ICT Board accessibility requirements and that all
three mandated languages (English, Kiswahili, and Somali) have complete translation coverage.

## Surfaces in scope

| Surface | Technology | Languages | Notes |
|---------|-----------|-----------|-------|
| portal-svc public portal | Astro + React | en, sw, so | Live tally and results display |
| kit Flutter app | Flutter 3 | en, sw, so | Voter verification workflow |
| field-mobile Flutter app | Flutter 3 | en, sw | Field officer workflow |
| ecc-svc admin | Django | en | Commission staff only |
| support-admin-svc | Django | en, sw | Helpdesk agents |

## Modes

### dry-mode

Validates that all ARB keys are present in all locale files and runs axe-core static analysis
against server-rendered HTML snapshots. No live browser session required.

```bash
just a11y-i18n dry
```

### live-mode

Launches Playwright-driven browsers against pre-prod services, runs axe-core on every page,
captures screenshots, and checks language switching.

```bash
just a11y-i18n live ENV=preprod
```

## WCAG checks performed

- Axe-core 4.x ruleset (WCAG 2.2 AA profile) on all routes
- Colour contrast ratio >= 4.5:1 for normal text, >= 3:1 for large text
- Focus visible on all interactive elements
- All images have non-empty `alt` text
- Form inputs have associated labels
- Dynamic content updates announced via ARIA live regions
- Keyboard navigation: full page operable without mouse
- Reduced-motion respected for animated tally counters

## i18n completeness checks

- All ARB keys present in `en`, `sw`, and `so` locale files
- No hardcoded English strings in Flutter widget tree (caught by `l10n_utils` lint)
- RTL layout tested for Somali (left-to-right language but flag for future Arabic support)
- Date and number formatting locale-aware (using `intl` package)
- Kiswahili and Somali translations reviewed by contracted native speakers (sign-off in
  `compliance/ipr-handover/translation-signoffs.yaml`)

## Expected artefacts

Written to `verification/reports/a11y-i18n-<date>/`:

- `axe-results.json` -- per-page axe-core violations
- `wcag-summary.html` -- human-readable WCAG compliance matrix
- `i18n-coverage.json` -- translation key coverage per locale
- `screenshots/` -- full-page screenshots in each language
- `gate-g6-a11y.sig` -- signed evidence for G6

## Pass criteria

- Zero axe-core violations at WCAG 2.2 AA level across all in-scope surfaces
- i18n coverage: 100% for `en` and `sw`, >= 98% for `so` (documented exceptions for proper nouns)
- Colour contrast: zero failures above 0-exception threshold
- Keyboard navigation: all critical paths completable without mouse

## Limitations

- Screen-reader compatibility (NVDA, VoiceOver) tested manually; not automated in this harness
- Print stylesheet accessibility not in scope
- Automated Somali translation coverage check does not validate translation quality

## Related docs

- `packages/eel-web-astro/` -- Astro base package
- `packages/kit-sdk-flutter/` -- Flutter shared components
- `compliance/elections-tech-regs/README.md` -- accessibility regulatory requirements
