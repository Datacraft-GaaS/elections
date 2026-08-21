# transparency (Astro site)

Transparency and reproducibility public site: transparency log, reproducibility bundles, bug bounty.

## Module

M14 (reproducibility-bundle) | transparency-svc | WP-2.1

## Purpose

Static Astro site (SSG, rebuilt on each new log entry) publishing transparency log
entries, reproducibility bundles, bug bounty programme information, and Merkle
inclusion proofs. Links are shared from the main public portal. Enables independent
verification of election result integrity by researchers, civil society, and the public.

## Locales

English, Swahili (2 locales).

## Content Collections

- `bundles`: reproducibility bundles (manifest + IPFS CID + DataCite DOI)
- `log-entries`: sequential transparency log entries with Merkle proof links
- `bounties`: bug bounty programme details and disclosure timeline

## Key Routes

| Route | Mode | Description |
|---|---|---|
| `/` | Static | Overview and latest log entry |
| `/log` | Static | Full log entry listing |
| `/log/{seq}` | Static | Single log entry with proof |
| `/bundles/{id}` | Static | Bundle detail with IPFS link and DOI |
| `/bundles/{id}/verify` | SSR | On-demand bundle verification |
| `/bounty` | Static | Bug bounty programme |

## Build / Run

```bash
cd services/transparency-svc/astro/transparency
pnpm install
pnpm dev
pnpm build
pnpm preview
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `TLOG_API_URL` | transparency-svc/crates/tlog-api (build-time fetch) |
| `IPFS_GATEWAY_URL` | IPFS public gateway for bundle links |
| `SITE_BASE_URL` | Canonical URL |
| `DATACITE_DOI_PREFIX` | DataCite prefix for DOI links |

## WCAG

2.2 AA.

## Deployment Notes

- Kubernetes: transparency-svc ingress
- Rebuild triggered by CI on each new log entry (eel.tlog.entry Kafka event)
- K8s manifest: services/transparency-svc/deploy/base/transparency-site-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M14)
- transparency-svc/crates/tlog-api
- integrations/ipfs
- integrations/datacite
- integrations/kbc-sftp (KBC broadcast publication)
- docs/runbooks/transparency-bundle-publish.md

## Upstream / Downstream

- Upstream: transparency-svc/crates/tlog-api, integrations/ipfs, integrations/datacite
- Downstream: portal-svc/astro/public (linked from portal), KBC SFTP broadcast
