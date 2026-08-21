# eel adhoc-editor (Astro)

Browser-side ad-hoc report authoring editor.

## Local dev

```bash
pnpm install
pnpm dev
```

Set `PUBLIC_ADHOC_API_URL` to point at the Rust adhoc API.
Use `EEL_MOCK_API=1` to start with a stub server instead.

## Tests

```bash
pnpm build
pnpm test
```

Playwright tests require a running dev server (`pnpm dev`).
If playwright/pnpm are not installed, tests are documented but skipped.
