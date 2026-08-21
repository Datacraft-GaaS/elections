# edge/rate-limits

Envoy Gateway sliding-window rate limit configuration for the eel edge layer. Limits are
applied per IP address, API key, bearer token, and endpoint combination before traffic
reaches Anubis, nginx-shield, or the core cluster.

## Purpose

Rate limiting is the first line of defense against request floods. It operates at the
Envoy Gateway layer, before WAF inspection or bot challenges, so it adds minimal latency
to legitimate traffic. Limits are tuned to allow full election-day kit and portal traffic
at 5x peak while blocking coordinated floods and credential-stuffing attempts.

Limits are stored in Valkey (shared across all rate-limit pods in a region). A request
that exceeds its limit receives a `429 Too Many Requests` response with a
`Retry-After` header calculated from the remaining window.

## Limit tiers

| Tier | Key dimension | Typical limit | Scope |
|------|--------------|---------------|-------|
| Global IP | Source IP | 300 req/min | All endpoints |
| Portal public | Source IP | 120 req/min | `/portal/*` |
| Results API | Source IP | 60 req/min | `/rts/results/*` |
| Kit API | Kit API key | 200 req/min per kit | BVI, RTS, BVR kit endpoints |
| Admin API | Bearer token | 30 req/min | Admin and reporting APIs |
| Voter lookup | Source IP + endpoint | 10 req/min | `/bvr/lookup/*` |

## Contents

| File | Description |
|------|-------------|
| `limits.yaml` | Envoy Gateway `RateLimitPolicy` definitions for all limit tiers |
| `burst-overrides.yaml` | Election-day burst overrides applied at T-30min before polls open |
| `exemptions.yaml` | Exempted CIDRs (monitoring, internal health checks, trusted scrapers) |
| `valkey-config.yaml` | Valkey connection config referenced by the rate limit filter |

## Usage

```bash
# Apply limit config to all edge clusters
for region in nairobi mombasa kisumu eldoret garissa; do
  kubectl apply -f edge/rate-limits/limits.yaml \
    --context=k8s-eel-$region -n eel-edge
done

# Apply election-day burst overrides (run at T-30min)
kubectl apply -f edge/rate-limits/burst-overrides.yaml \
  --context=k8s-eel-nairobi -n eel-edge
# ... repeat for other regions

# Check current rate limit counters in Valkey
kubectl exec -n eel-edge deploy/valkey -- \
  redis-cli --scan --pattern 'ratelimit:*' | head -20
```

## Config and environment

| Key | Value | Description |
|-----|-------|-------------|
| `VALKEY_ADDR` | `valkey.eel-edge.svc.cluster.local:6379` | Valkey address for limit counters |
| `RATE_LIMIT_WINDOW` | `60s` | Default sliding window |
| `RATE_LIMIT_BURST_FACTOR` | `2.0` | Burst multiplier for election-day override |
| Limit definitions | `limits.yaml` | Envoy `RateLimitPolicy` resources |

## Dependencies

- Envoy Gateway: executes the `RateLimitPolicy` CRDs defined here
- Valkey: distributed counter store (shared with Anubis token cache in same instance)
- `edge/anubis/`: bot challenge layer downstream; receives traffic that passes rate limits

## Test and verify

```bash
# Flood a rate-limited endpoint; expect 429 after threshold
for i in $(seq 1 150); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    https://portal.eel.local/results/2027/presidential
done | sort | uniq -c
# Expected: ~120 lines of 200, ~30 lines of 429

# Confirm Retry-After header on 429
curl -I https://portal.eel.local/results/2027/presidential \
  -H "X-Forwarded-For: 192.0.2.1" | grep -i retry-after
```

## Related docs

- `edge/anubis/README.md`: bot challenge layer downstream of rate limiting
- `edge/nginx/README.md`: origin-shield downstream
- `edge/README.md`: edge architecture overview
- `docs/adr/0019-edge-topology.md`: topology and rate-limit design decisions

## Limitations

- Valkey is a single logical instance per region; its failure disables rate limiting.
  Envoy Gateway falls back to `allow-all` mode on Valkey unavailability. Monitor Valkey
  health as a top-of-stack alert.
- Election-day burst overrides must be applied manually at T-30min. Automate via the
  election-day runbook (`docs/runbooks/election-day-ops.md`).
- IPv6 source addresses are tracked at /64 prefix granularity to avoid per-address
  counter explosion.
