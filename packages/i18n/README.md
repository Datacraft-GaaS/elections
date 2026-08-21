# @eel/i18n

ICU MessageFormat locale files for the eel (IEMS) citizen portal. Covers 11 languages used in Kenya.

## Locales

| Code | Language      | Script  |
|------|---------------|---------|
| en   | English       | Latin   |
| sw   | Swahili       | Latin   |
| ki   | Kikuyu        | Latin   |
| luo  | Dholuo        | Latin   |
| luy  | Luhya         | Latin   |
| kln  | Kalenjin      | Latin   |
| kam  | Kamba         | Latin   |
| mer  | Meru          | Latin   |
| guz  | Gusii         | Latin   |
| so   | Somali        | Latin   |
| oro  | Oromo         | Latin   |

## Files per locale

Each locale directory contains six JSON files with ICU MessageFormat strings:

- `common.json` - Shared UI chrome: nav, footer, buttons, date/number formats
- `voter.json` - Voter registration check, voter register, transfers, objections
- `results.json` - Election results display, tallying, verification
- `grievance.json` - Grievance filing, tracking, status
- `errors.json` - Error messages, field validation
- `accessibility.json` - ARIA labels and screen reader strings

## Font stacks

- **Somali (so)**: Uses Latin script (standard for Kenya). Font stack: `'Noto Sans', sans-serif`. Osmanya script (`'Noto Sans Osmanya', 'Noto Sans Arabic', sans-serif`) is not used in this deployment.
- **Oromo (oro)**: Uses Latin script (Qubee). Font stack: `'Noto Sans', sans-serif`. Ge'ez script Oromo is not used in Kenya; Qubee (Latin) is standard.
- **All other locales**: Latin script. Font stack: `'Noto Sans', 'Segoe UI', sans-serif`.

## Tools

```sh
# Check all non-English locales have every key present in English
node tools/i18n-lint.mjs

# Generate pseudo-locale (locales/pseudo/) for UI string expansion testing
node tools/pseudo-gen.mjs
```

## ICU MessageFormat

Values use ICU MessageFormat syntax. Examples:

```
{count, plural, =0 {No voters found} =1 {1 voter found} other {{count} voters found}}
{position, selectordinal, one {#st} two {#nd} few {#rd} other {#th}} place
{date, date, long}
{value, number, percent}
```

Pseudo-locale generation preserves ICU placeholders (`{...}`) intact and accents all literal characters.

## Non-English translations

The 10 non-English locales currently contain English placeholder values. Replace with actual translations before production use. The lint tool (`i18n-lint.mjs`) enforces key completeness: every key present in `en/` must appear in each other locale.
