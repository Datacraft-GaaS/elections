# design-system

Shared design tokens, components, and iconography for all eel portal and admin surfaces: Astro, Svelte, and Django. WCAG 2.2 AA compliant, colour-blind safe, dark-mode ready.

## Purpose in context

The system serves voters, Commission staff, oversight observers, and field officers across 46,229 polling stations. Every surface must look and behave consistently. This package provides the single source of design truth: a `tokens.json` design token file and framework-specific component libraries that consume it.

## Layout

```
design-system/
  tokens.json          # Design Tokens Community Group format; all colours, spacing, typography
  astro/               # Astro component library (used by eel-web-astro and portal sites)
  svelte/              # Svelte component library (used by LiveView-adjacent Svelte islands)
  django/              # Django template tags and static assets (used by eel-web-py)
  README.md
```

## Token structure

```json
{
  "color": {
    "brand": {
      "primary": { "light": "#1a56db", "dark": "#4f83f1", "wcag": "AA" },
      "secondary": { "light": "#0e9f6e", "dark": "#31c48d", "wcag": "AA" }
    },
    "neutral": { "50": ..., "100": ..., "900": ... }
  },
  "spacing": { "1": "4px", "2": "8px", ... },
  "typography": { "body": { "size": "16px", "lineHeight": "1.5" }, ... }
}
```

All colour pairs have been verified at >= 4.5:1 contrast ratio (AA) in both light and dark modes.

## Consuming in Astro

```astro
---
import { Button, StatusBanner, DataTable } from '@eel/design-system/astro';
---
<Button variant="primary">{t('common.submit', locale)}</Button>
<StatusBanner type="success">{t('common.saved', locale)}</StatusBanner>
```

## Consuming in Django templates

```html
{% load eel_design %}
{% eel_button variant="primary" %}{{ _("common.submit") }}{% endeel_button %}
```

## Consuming in Svelte

```svelte
<script>
  import { Badge, Alert } from '@eel/design-system/svelte';
</script>
<Alert type="warning">...</Alert>
```

## Install

```bash
# Astro / Svelte:
npm install @eel/design-system

# Django (copies static assets):
pip install eel-design-system
python manage.py collectstatic
```

## Test

```bash
# Token format validation:
node design-system/tools/validate-tokens.js

# Contrast ratio checks:
node design-system/tools/check-contrast.js

# Component visual regression (Playwright):
just visual-test-design-system
```

## Related docs

- `packages/eel-web-astro/README.md` - Astro integration that uses this package
- `packages/eel-web-py/README.md` - Django integration
- `packages/i18n/README.md` - locale strings used in component labels
- `docs/adr/0015-design-system.md` - design system rationale and accessibility requirements

## Limitations

The Svelte component library is a thin wrapper for use in LiveView Svelte islands; it does not include a full Svelte routing or state management layer. The Django template tags require `eel-web-py` to be installed. Dark mode is implemented via CSS custom properties; browsers that do not support `prefers-color-scheme` will receive the light theme.
