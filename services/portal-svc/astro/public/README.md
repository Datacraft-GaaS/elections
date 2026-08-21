# public (Astro site)

Main public election portal: signed result publications, Merkle proofs, candidate info, and turnout data.

## Module

M11 (public-portal) | portal-svc | WP-3.9

## Purpose

Primary public-facing Astro site for election results and transparency. Displays signed
Form 34 result publications, Merkle tree inclusion proofs for individual forms,
live turnout updates, candidate profiles, and polling station information. Designed
for very high peak traffic (5 million requests per hour) via heavy CDN caching.
SSR is used only for on-demand Merkle proof generation; all result display is static.

## Locales

11 Kenyan languages. URL prefix by locale code.

## Content Collections

- `results`: published Form 34A/B/C summaries per constituency
- `candidates`: candidate profiles per election and constituency
- `polling-stations`: station lookup data (updated T-30 days before election)
- `faqs`: election process FAQ

## Key Routes

| Route | Mode | Description |
|---|---|---|
| `/` | Static | National results overview |
| `/results/{election_id}` | Static | Election-level results |
| `/results/{election_id}/constituency/{code}` | Static | Constituency results with Form 34 scan |
| `/verify/{hash}` | SSR | Merkle inclusion proof on-demand |
| `/candidates` | Static | Candidate listing |
| `/polling-stations` | Static | Station lookup |

## Build / Run

```bash
cd services/portal-svc/astro/public
pnpm install
pnpm dev
pnpm build    # outputs to dist/
pnpm preview
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `PORTAL_API_URL` | portal-svc/crates/portal-api (build-time data fetch) |
| `TRANSPARENCY_API_URL` | transparency-svc (Merkle proof endpoint) |
| `SITE_BASE_URL` | Canonical URL for sitemaps and OG tags |
| `CDN_PURGE_URL` | CDN purge endpoint (post-publication rebuild hook) |

## WCAG

2.2 AA. All result tables have ARIA labels. Charts have text equivalents.

## Deployment Notes

- Kubernetes: portal-svc ingress, CDN layer (Cloudflare or equivalent) in front
- Rebuild triggered on each new result publication event (eel.rts.publications Kafka)
- K8s manifest: services/portal-svc/deploy/base/public-site-deployment.yaml
- CDN purge on rebuild via CDN_PURGE_URL hook

## SLA

p95 200ms, p99 500ms at 5M requests/hour (mostly CDN-served).

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M11)
- portal-svc/crates/portal-api
- transparency-svc/crates/tlog-api (Merkle proofs)
- portal-svc/ex/portal_live (real-time LiveView backend)
- docs/runbooks/portal-publication.md

## Upstream / Downstream

- Upstream: portal-svc/crates/portal-api, transparency-svc (Merkle proofs)
- Downstream: none (public read-only)
