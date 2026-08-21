# hello_site (Astro site)

Hello-world Astro site used as scaffold template and CI smoke-test for packages/eel-web-astro.

## Purpose

Scaffolding template and CI validation for the `packages/eel-web-astro` shared Astro
base package. Not a production site. Demonstrates minimum viable Astro configuration
with eel-web-astro theme, i18n stub, and CI pipeline integration.

## Build / Run

```bash
cd services/hello-svc/astro/hello_site
pnpm install
pnpm dev
pnpm build
```

## Notes

- Template only. Do not deploy to production Kubernetes ingress.
- Used only for CI pipeline validation of eel-web-astro package changes.
