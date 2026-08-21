# edge/anubis

Proof-of-work bot mitigation and escalation ladder for the eel edge layer. Anubis is a
Go service that issues CPU-bound challenges to suspected bot traffic before requests reach
the portal, results, and voter-verification endpoints.

## Purpose

Election results pages and voter lookup endpoints attract scraping and coordinated
request floods during elections. Anubis sits between Envoy Gateway and nginx-shield to
intercept traffic that fails heuristic checks and issues a JavaScript-based PoW challenge.
Human browsers solve the challenge in under a second; bots either fail or consume CPU.

The escalation ladder has three levels:

1. `PASS`: request looks legitimate, forwarded without challenge.
2. `CHALLENGE`: browser gets a PoW puzzle (SHA-256 partial preimage, difficulty tuned per
   endpoint). Solved token is cached in Valkey for 10 minutes.
3. `BLOCK`: source IP or ASN exceeds block threshold after repeated challenge failures.

Kit firmware and official apps carry a static bypass token issued per firmware build.
The token is a signed JWT with a `kit` claim; Anubis verifies the signature and passes
the request without issuing a challenge.

## Contents

| File | Description |
|------|-------------|
| `config.yaml` | Main Anubis config: difficulty levels, endpoint rules, bypass token public key |
| `endpoints.yaml` | Per-endpoint heuristic rules: rate ceiling, challenge trigger, block threshold |
| `kit-bypass-pubkey.pem` | Public key for verifying kit firmware bypass JWTs |
| `difficulty-tuning.md` | Reasoning for per-endpoint difficulty settings |

## Usage

```bash
# Reload config without restart (Anubis supports SIGHUP-based config reload)
kubectl exec -n eel-edge deploy/anubis -- kill -HUP 1

# Check current challenge stats
kubectl exec -n eel-edge deploy/anubis -- anubis-stats

# Test that a legit browser request gets a challenge page (not a 403)
curl -s https://portal.eel.local/results/2027/presidential | grep "anubis-challenge"

# Test that a kit bypass token passes without challenge
KIT_TOKEN="$(cat /path/to/kit-bypass.jwt)"
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $KIT_TOKEN" \
  https://bvi.eel.local/identify
# Expected: 200 (or forwarded to upstream; not a challenge page)
```

## Config and environment

| Key | Default | Description |
|-----|---------|-------------|
| `ANUBIS_VALKEY_ADDR` | `valkey.eel-edge.svc.cluster.local:6379` | Token cache backend |
| `ANUBIS_DIFFICULTY` | per endpoint in `endpoints.yaml` | SHA-256 difficulty level |
| `ANUBIS_CHALLENGE_TTL` | `600` | Solved token cache TTL in seconds |
| `ANUBIS_KIT_BYPASS_KEY` | path to `kit-bypass-pubkey.pem` | Public key for kit JWT verification |
| `ANUBIS_BLOCK_THRESHOLD` | `10` | Challenge failures before IP block |

Garissa cluster uses extended timeout config (`ANUBIS_CHALLENGE_TIMEOUT=30s`) to
accommodate satellite-path latency.

## Dependencies

- Valkey: stores solved challenge tokens and block lists
- Envoy Gateway: routes suspicious traffic to Anubis via `HTTPRoute` match rules
- Kit firmware: carries a signed bypass JWT embedded at build time
- `edge/rate-limits/`: Envoy rate limiting complements Anubis; heavy floods are rate-limited
  before reaching Anubis

## Test and verify

```bash
# Confirm challenge is served to plain curl (no bypass token)
curl -s https://portal.eel.local/results | grep -i anubis

# Confirm blocked IP returns 429 after repeated failures
# (do not run in prod; use staging cluster)

# Anubis pod health
kubectl get pod -n eel-edge -l app=anubis
kubectl logs -n eel-edge -l app=anubis --tail=20
```

## Related docs

- `edge/rate-limits/README.md`: Envoy rate limiting layer upstream of Anubis
- `edge/nginx/README.md`: origin-shield nginx downstream
- `edge/README.md`: edge architecture overview
- `docs/THREAT-MODEL.md`: bot and scraping threat model

## Limitations

- Anubis challenges do not work for clients without JavaScript. The public results
  portal falls back to a minimal non-JS page if the challenge cannot be presented.
- Kit bypass tokens are per firmware build; compromised tokens require a firmware
  revocation list update and redeployment of `kit-bypass-pubkey.pem`.
- Satellite-path clients (Garissa) may time out on the default challenge difficulty;
  the extended timeout config must be applied before election day.
