# eel-web-astro

Shared Astro integration for eel public-facing and oversight portal sites: signed-content loader, Swahili/English i18n scaffolding, SEO helpers, and authentication token stubs.

## Purpose in context

Astro-based portal sites in `services/<svc>/astro/<site>/` install this package to get consistent content loading (with transparency-log signature verification), internationalization, and SEO metadata. It is the Astro counterpart to `eel-web-py` and references the same `design-system` tokens.

## Public exports

```typescript
// packages/eel-web-astro/src/index.ts
export * from './loaders/SignedContentLoader.js';
export * from './i18n/index.js';
export * from './seo/index.js';
export { default as integration } from './integration.js';
```

| Export | Description |
|---|---|
| `SignedContentLoader` | Astro content loader that verifies Ed25519 signatures on fetched content against the transparency log |
| `i18n` | Type-safe Swahili/English translation lookup; `t('key', locale)` |
| `seo` | `<EelSeoMeta>` Astro component; injects canonical URL, OG tags, structured data |
| `integration` | Astro integration; adds CSP headers, cache directives, and signed-content middleware |

## Layout

```
eel-web-astro/
  src/
    loaders/SignedContentLoader.ts
    i18n/index.ts
    seo/index.ts
    integration.ts
    index.ts
  tests/
  node_modules/
  package.json
  tsconfig.json
  vitest.config.ts
```

## Usage

```typescript
// astro.config.mjs
import { defineConfig } from 'astro/config';
import eelWeb from '@eel/web-astro';

export default defineConfig({
  integrations: [eelWeb({ signatureKeyUrl: '/api/log/keys' })],
});
```

```astro
---
// src/pages/results/[stationId].astro
import { t } from '@eel/web-astro';
import { EelSeoMeta } from '@eel/web-astro';
const locale = Astro.locals.locale ?? 'sw';
---
<EelSeoMeta title={t('results.title', locale)} />
<h1>{t('results.heading', locale)}</h1>
```

## Config

| Option | Description |
|---|---|
| `signatureKeyUrl` | URL to fetch the Ed25519 verifying key for signed content |
| `defaultLocale` | `'sw'` (Swahili) or `'en'` (English); default `'sw'` |
| `cspNonce` | CSP nonce generator function; default is a random 128-bit hex |

## Install

```bash
npm install @eel/web-astro
# or from source:
npm install --workspace packages/eel-web-astro
```

## Test

```bash
npm --workspace packages/eel-web-astro run test
npm --workspace packages/eel-web-astro run typecheck
```

## Related docs

- `packages/design-system/README.md` - shared design tokens used in templates
- `packages/i18n/README.md` - locale files for Swahili and English
- `packages/openapi-contracts/README.md` - API specs for the signed-content endpoints
- `packages/verifier-rs/README.md` - WASM verifier optionally used by `SignedContentLoader`

## Limitations

`SignedContentLoader` performs signature verification at build time for static content and at request time for SSR routes. Build-time verification requires network access to the transparency log; CI pipelines must allow this outbound connection or use a cached key snapshot.
