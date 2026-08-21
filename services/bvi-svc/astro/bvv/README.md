# bvv (Astro site)

Public-facing voter verification and polling station lookup site.

## Module

M07 (bvv - biometric voter verification public lookup) | bvi-svc | WP-3.7

## Purpose

Hybrid Astro site where citizens check their voter registration status and polling station
assignment. The verification widget delegates to bvi-svc/ex/bvv_web (Phoenix LiveView)
for the server-side query. Anti-enumeration controls are enforced: CAPTCHA on the lookup
form, one successful lookup per IP per 10 minutes (Redis rate limit), no bulk export.
Static pages (FAQ, polling station locator) are SSG.

## Locales

11 Kenyan languages. URL prefix by locale code (`/sw/`, `/en/`, `/ki/`, ...).

## Content Collections

- `faq`: voter registration FAQ in 11 languages
- `polling-station-locator`: static constituency-to-station mapping

## Key Routes

| Route | Mode | Description |
|---|---|---|
| `/` | Static | Landing page |
| `/verify` | SSR | Voter lookup form (CAPTCHA-gated) |
| `/find-station` | Static | Polling station search by constituency |
| `/faq` | Static | FAQ by language |

## Build / Run

```bash
cd services/bvi-svc/astro/bvv
pnpm install
pnpm dev      # development server
pnpm build    # production SSG+SSR build
pnpm preview  # preview production build
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `BVI_LIVE_URL` | bvi-svc/ex/bvv_web LiveView origin for SSR proxy |
| `SITE_BASE_URL` | Canonical site URL |
| `RECAPTCHA_SITE_KEY` | reCAPTCHA v3 site key |
| `RATE_LIMIT_WINDOW_SECS` | Anti-enumeration window (default 600) |

## WCAG

2.2 AA. High-contrast mode supported. All interactive elements keyboard-navigable.

## Anti-Enumeration Controls

- CAPTCHA on every lookup (reCAPTCHA v3 score >= 0.7)
- Max 1 successful lookup per IP per 10 minutes (Redis, enforced in bvv_web)
- No API endpoint exposed without CAPTCHA

## Deployment Notes

- Kubernetes: bvi-svc ingress (heavy CDN caching for static pages, pass-through for SSR)
- K8s manifest: services/bvi-svc/deploy/base/bvv-site-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M07)
- bvi-svc/ex/bvv_web (LiveView backend)
- bvi-svc/crates/bvi-api
- docs/runbooks/bvv-anti-enum.md

## Upstream / Downstream

- Upstream: bvi-svc/ex/bvv_web (voter lookup), bvi-svc/crates/bvi-api
- Downstream: none (public read-only)
