# edge/varnish

Long-TTL signed-snapshot caching for static election assets in the eel edge layer.
Varnish sits between nginx-shield and the core K8s cluster, absorbing repeat reads of
candidate lists, constituency maps, polling station directories, and public results pages.

## Purpose

Election assets have two distinct cacheability profiles:

1. Pre-election static assets (candidate lists, polling station directories, boundary maps):
   long TTL (up to 24h), cache until an explicit PURGE or TTL expiry.
2. Live results pages: short TTL (30s), cache with `stale-while-revalidate` for result
   flood absorption during election night.

Varnish enforces these TTLs via VCL rules keyed on service and endpoint. Cached responses
carry an `X-Eel-Cache: HIT|MISS|PASS` header for observability.

Snapshot signing: static asset responses include a `X-Eel-Snapshot-Sig` header set by the
upstream service. Varnish caches the signature alongside the body so downstream clients can
verify authenticity even on a cache hit.

## Contents

| File | Description |
|------|-------------|
| `default.vcl` | Main VCL: backend definitions, TTL rules, PURGE ACL, header passthrough |
| `acl-purge.vcl` | IP ACL for PURGE requests (internal cluster CIDRs only) |
| `backends.vcl` | Backend pool definitions pointing to nginx-shield on each cluster |

## Usage

```bash
# Purge a cached path (run from within the cluster or from an allowed CIDR)
curl -X PURGE https://portal.eel.local/candidates/2027

# Check cache status of a response
curl -I https://portal.eel.local/polling-stations | grep x-eel-cache

# Reload VCL without restarting (Varnish Management Interface)
varnishadm -T varnish.eel-edge.svc.cluster.local:6082 \
  vcl.load v2 /etc/varnish/default.vcl && \
  varnishadm -T varnish.eel-edge.svc.cluster.local:6082 vcl.use v2
```

## Config and environment

| Setting | Value | Description |
|---------|-------|-------------|
| `VARNISH_HTTP_PORT` | `6081` | HTTP listen port |
| `VARNISH_MGMT_PORT` | `6082` | Management interface port |
| `VARNISH_STORAGE` | `malloc,512m` | In-memory storage, 512MB per pod |
| `VARNISH_TTL` | set per VCL rule | Not a global env var |
| Purge ACL CIDR | cluster internal CIDRs | Defined in `acl-purge.vcl` |

## Dependencies

- nginx-shield: Varnish backends point to nginx on `127.0.0.1:8080` (sidecar) or the
  nginx service in the cluster
- Upstream services must set correct `Cache-Control` and `X-Eel-Snapshot-Sig` headers
- Valkey (shared with Envoy Gateway rate limiting) is NOT used by Varnish; Varnish state
  is in-process only

## Test and verify

```bash
# Confirm HIT on repeat request
curl -I https://portal.eel.local/polling-stations | grep x-eel-cache
curl -I https://portal.eel.local/polling-stations | grep x-eel-cache
# First: MISS, second: HIT

# Confirm snapshot signature is preserved on HIT
curl -I https://portal.eel.local/candidates/2027 | grep x-eel-snapshot-sig

# Varnish stats
kubectl exec -n eel-edge deploy/varnish -- varnishstat -1 -f cache_hit,cache_miss
```

## Related docs

- `edge/nginx/README.md`: upstream origin-shield layer
- `edge/README.md`: edge architecture overview
- `docs/adr/0019-edge-topology.md`: edge topology ADR

## Limitations

- Varnish state is per-pod and in-memory. Pod restarts cold-start the cache, causing a
  miss storm for approximately 5 minutes on election night if a pod restarts.
- PURGE requests are restricted to internal CIDRs; coordinate with the network team before
  issuing bulk PURGEs from non-cluster scripts.
- Results pages with short TTLs (30s) do not benefit from caching during peak flood; the
  upstream `rts-svc` must handle the residual traffic directly.
