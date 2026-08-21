# eel-public-reports

Static Astro 5 site that publishes ABIS public transparency reports for citizens, media, and observers.

## Prerequisites

- [pnpm](https://pnpm.io) >= 8 (preferred)
- Node.js >= 20

If pnpm is not on PATH, install it via `npm i -g pnpm` or use `npx pnpm@latest`.

## Build

```sh
# Install dependencies
pnpm install

# Fetch latest published reports from the API (optional; falls back to committed placeholder)
REPORTS_API_URL=https://reports.eel.internal pnpm fetch-published

# Build static site
pnpm build

# Preview
pnpm preview
```

## Tests

Tests use Playwright. The webServer config in `playwright.config.ts` starts `pnpm preview` automatically.

```sh
# Install playwright browsers on first run
pnpm exec playwright install --with-deps chromium

# Run all tests (builds must exist: run pnpm build first)
pnpm test
```

If Playwright or pnpm are not available, build can be verified by running `pnpm build` and checking `dist/` for the expected files:
- `dist/index.html`
- `dist/en/reports/public-fmr-fnmr-monthly/00000000-0000-7000-8000-000000000001/index.html`
- `dist/api/rss.xml`
- `dist/api/atom.xml`

## Locales

The site supports 11 locales: `en sw ki luo kal kam guj mer mas ksl-glossary som`. The default locale is `en` (no URL prefix). All other locales are prefixed.

## Fetch published script

`src/scripts/fetch-published.mjs` pulls public-class reports from the REST API and writes `src/data/reports.json`. Run it before building to get fresh data.

```sh
REPORTS_API_URL=http://reports-svc:8080 \
REPORTS_TOKEN=<svid-jwt> \
pnpm fetch-published
```

If the API is unavailable the script exits 0 and the build uses the committed placeholder.
