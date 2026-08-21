# A11y + i18n Report: public-portal

Generated: 2026-08-21T00:00:00Z
Surface: public-portal (M11)
Service: portal-svc
Framework: astro
WCAG target: AA

## Axe Violations

Zero violations.

## Pseudo-locale (xxa)

Status: pass
No truncation issues detected.

## Locale completeness

| Locale | Completeness | Font | ICU plurals | RTL |
|---|---|---|---|---|
| en-KE | 100% | ok | ok | n/a |
| sw-KE | 100% | ok | ok | n/a |
| om-ET | 95% | ok | ok | n/a |
| so-KE | 95% | ok | ok | n/a |
| kam-KE | 90% | ok | ok | n/a |
| mer-KE | 90% | ok | ok | n/a |
| guz-KE | 88% | ok | ok | n/a |
| luo-KE | 90% | ok | ok | n/a |
| luy-KE | 88% | ok | ok | n/a |
| kln-KE | 85% | ok | ok | n/a |
| mas-KE | 85% | ok | ok | n/a |

Note: RTL mechanism (dir attribute + logical CSS) tested via pseudo-locale xxa; not required for these locales.

## Screen-reader script results

| Platform | Tool | Script | Status | Notes |
|---|---|---|---|---|
| Windows | NVDA 2024.1 | nvda-scripts/portal.py | pass | All landmarks and headings navigable |
| Android | TalkBack 14 | talkback-scripts/citizen-verify.sh | pass | NIN input field correctly labelled |
| macOS | VoiceOver | voiceover-scripts/ro-desktop.applescript | pass | First element described; heading navigable |

## Summary

PASS: surface meets AA conformance. All locale and screen-reader checks green.
