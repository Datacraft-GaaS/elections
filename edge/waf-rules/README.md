# edge/waf-rules

Coraza WAF rule sets for the eel edge layer. These are tuned per-service rules, not the
default CRS (OWASP Core Rule Set). They represent the output of threat-modeling the 18
election services against Kenya's election-specific attack surface.

## Purpose

The default CRS produces too many false positives for election APIs: biometric payloads,
signed JSON+CBOR bodies, and kit firmware requests all trigger generic injection rules.
This directory contains hand-tuned rule sets that:

1. Block known attack patterns relevant to elections (SQL injection in voter lookup, XSS
   on results pages, path traversal on static assets).
2. Allow legitimate kit traffic (CBOR bodies, large biometric blobs, bearer tokens).
3. Apply service-specific rules per the `Host` and `X-Eel-Service` routing headers.

## Contents

| File | Description |
|------|-------------|
| `rules/portal.conf` | Rules for the public portal (`portal-svc`): SQLi, XSS, scraping |
| `rules/rts.conf` | Rules for results transmission: replay detection, oversized bodies |
| `rules/bvi.conf` | Rules for biometric identification: allowlist kit user-agents, block replay |
| `rules/bvr.conf` | Rules for voter registration: SQLi on search, enumeration rate guard |
| `rules/public.conf` | Rules for public static assets: path traversal, hotlinking |
| `exclusions/kit-agents.conf` | Allowlisted kit firmware user-agent strings |
| `exclusions/cbor-content-types.conf` | Exclusions for `application/cbor` content types |
| `tuning/thresholds.conf` | Anomaly score thresholds per service |

## How to update

1. Edit the relevant rule file.
2. Test in paranoia level 1 against the staging edge cluster using a synthetic request set.
3. Review the anomaly score delta in `reports/` before promoting to prod.

```bash
# Validate Coraza config syntax (run from the nginx build image)
docker run --rm \
  -v "$(pwd)/edge/waf-rules:/etc/coraza-rules:ro" \
  -v "$(pwd)/edge/nginx:/etc/nginx:ro" \
  ghcr.io/eel-project/nginx-coraza:latest nginx -t

# Tail WAF logs during a test run
kubectl logs -n eel-edge -l app=nginx-shield --follow | grep 'coraza'
```

## Config and environment

| Setting | Location | Description |
|---------|----------|-------------|
| `SecRuleEngine` | `portal.conf` header | `On` in prod, `DetectionOnly` in staging |
| `SecAnomalyThreshold` | `tuning/thresholds.conf` | Inbound anomaly score ceiling per service |
| `SecRequestBodyLimit` | per-service conf | Raised to 10MB for biometric endpoints |
| Rule IDs | `9000000-9099999` | eel-specific rule namespace (above CRS range) |

## Dependencies

- Coraza WAF library embedded in the nginx build image (`ghcr.io/eel-project/nginx-coraza`)
- nginx-shield loads rules via `SecIncludeRules` directives in `edge/nginx/conf.d/coraza.conf`
- Threat model: `docs/THREAT-MODEL.md` per service; WAF rules map to the threat IDs there

## Test and verify

```bash
# SQLi probe against portal (expect 403)
curl -s -o /dev/null -w "%{http_code}" \
  "https://portal.eel.local/search?q='; DROP TABLE voters;--"

# Legit kit request should pass (expect 200)
curl -s -o /dev/null -w "%{http_code}" \
  -H "User-Agent: eel-kit-fw/3.1" \
  -H "Content-Type: application/cbor" \
  https://bvi.eel.local/identify

# Check anomaly score header on a borderline request
curl -I https://portal.eel.local/results | grep x-coraza-score
```

## Related docs

- `edge/nginx/README.md`: how rules are loaded
- `docs/THREAT-MODEL.md`: per-service threat models
- `edge/README.md`: edge layer overview

## Limitations

- Rule updates require a pod rolling restart (ConfigMap hash annotation triggers it).
- Rules are not hot-reloaded; a misconfigured rule blocks all traffic through that service
  until the pod restarts with the corrected config. Test in staging first.
- Kit firmware user-agent strings must be kept in sync with `exclusions/kit-agents.conf`
  when firmware versions are released.
