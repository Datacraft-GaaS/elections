# i18n

Shared Swahili and English locale files for all eel surfaces: kit apps, portals, and admin interfaces.

## Purpose in context

All user-facing text in the system must be available in both Swahili (`sw`) and English (`en`). This package provides the canonical locale files consumed by the Flutter SDK (`intl`), Astro portals (`eel-web-astro`), and Django admin (`eel-web-py`). Translators update files here; no service-level duplication is permitted.

## Layout

```
i18n/
  locales/
    sw/
      common.json       # shared labels, buttons, errors
      rts.json          # RTS surface strings
      bvr.json          # BVR surface strings
      portal.json       # voter portal strings
      kit.json          # kit device UI strings
      # ... one file per bounded context
    en/
      common.json
      rts.json
      # ... mirrors of sw/
  tools/
    check_missing.sh    # reports keys present in one locale but absent in the other
    extract_keys.sh     # extracts keys from source files for translator workflow
  package.json
  README.md
```

## Key conventions

- Keys are dot-separated: `common.save`, `rts.form34a.submit_button`
- Values are plain strings; no ICU message format plurals (use separate keys: `item`, `item_plural`)
- No HTML in locale strings; use wrapper components in the UI layer
- Swahili is the primary locale; English is secondary

## Usage in Flutter

```dart
// pubspec.yaml: intl package + flutter gen-l10n
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

Text(AppLocalizations.of(context)!.rtsTallySubmitButton)
```

## Usage in Astro

```typescript
import { t } from '@eel/web-astro';
const label = t('rts.form34a.submit_button', 'sw');
```

## Usage in Django templates

```html
{% load eel_i18n %}
{% trans "rts.form34a.submit_button" %}
```

## Validate

```bash
# Check for missing keys between locales:
bash i18n/tools/check_missing.sh

# Check all JSON files are valid:
node -e "require('fs').readdirSync('i18n/locales/sw').forEach(f => JSON.parse(require('fs').readFileSync('i18n/locales/sw/' + f)))"
```

## Adding a new string

1. Add the key to both `locales/sw/<context>.json` and `locales/en/<context>.json`
2. Run `bash i18n/tools/check_missing.sh` to confirm parity
3. Run Flutter `flutter gen-l10n` if the string appears in kit apps

## Related docs

- `packages/kit-sdk-flutter/README.md` - Flutter i18n configuration
- `packages/eel-web-astro/README.md` - Astro i18n usage
- `packages/eel-web-py/README.md` - Django i18n integration

## Limitations

Machine translation has not been reviewed for election-specific terminology. All Swahili strings must be reviewed by a native speaker with elections domain knowledge before deployment. Legal strings (declarations, oaths) must additionally be reviewed by the Commission's legal team.
