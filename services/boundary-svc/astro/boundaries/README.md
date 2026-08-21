# boundaries (Astro site)

Public-facing boundary transparency site displaying electoral boundary maps, fairness metrics, and change history.

## Module

M18 (boundary-transparency) | boundary-svc | WP-2.4

## Purpose

Static Astro site (SSG, daily rebuild) that gives citizens and researchers access to
electoral boundary information: interactive SVG/WebGL constituency maps, fairness
metrics (population deviation, compactness scores), boundary change history, and
geofence polygons for kit GPS validation. Sourced from NLC and KNBS shapefiles ingested
by boundary-svc.

## Locales

English, Swahili (2 locales). URL prefix: `/en/`, `/sw/`.

## Content Collections

- `boundary-changes`: gazette-referenced boundary amendments with before/after polygons
- `fairness-reports`: per-election-cycle fairness metric snapshots
- `faqs`: voter-facing explanations of boundary delimitation

## Key Routes

| Route | Description |
|---|---|
| `/` | Landing with national map overview |
| `/boundaries/{code}` | Constituency detail: map, population, history |
| `/changes` | List of all boundary changes with gazette reference |
| `/metrics` | Fairness metric dashboard (population deviation etc.) |
| `/objections` | (SSR) Citizen objection submission form |

## Build / Run

```bash
cd services/boundary-svc/astro/boundaries

# Install dependencies
pnpm install

# Development server (hot reload)
pnpm dev

# Production build (outputs to dist/)
pnpm build

# Preview production build locally
pnpm preview
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `BOUNDARY_API_URL` | boundary-svc API for data fetch at build time |
| `PUBLIC_MAPBOX_TOKEN` | Mapbox GL JS token (if used, else inline SVG) |
| `SITE_BASE_URL` | Canonical site URL for sitemaps |

## WCAG

2.2 AA. All maps must have text equivalents (data tables) for screen reader users.

## Deployment Notes

- Kubernetes: boundary-svc ingress (CDN-fronted, static files)
- Rebuilt daily by CI job (boundary-rebuild pipeline) to pick up shapefile updates
- K8s manifest: services/boundary-svc/deploy/base/boundaries-site-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M18)
- boundary-svc/crates/boundary-api
- docs/runbooks/boundary-svc.md

## Upstream / Downstream

- Upstream: boundary-svc/crates/boundary-api (boundary data), NLC/KNBS shapefiles
- Downstream: none (public read-only site)
