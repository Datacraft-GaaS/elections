# edge/nginx

Origin-shield nginx configuration for the eel edge layer. Nginx sits between Envoy Gateway
and Varnish, handling TLS termination, upstream routing, and connection-level hardening.

## Purpose

Nginx in this context is an origin-shield, not a general-purpose web server. Its job is to:

1. Terminate TLS for traffic arriving from Envoy Gateway.
2. Route requests to the correct upstream service pool based on `Host` and `X-Eel-Service` headers.
3. Enforce connection-level hardening (cipher suites, HSTS, client certificate optional).
4. Pass traffic to Varnish for caching before hitting the core K8s cluster.
5. Apply Coraza WAF inspection inline (Coraza is embedded as an nginx module).

## Contents

| File | Description |
|------|-------------|
| `nginx.conf` | Main nginx configuration: worker tuning, event model, global TLS defaults |
| `conf.d/upstream-map.conf` | `map` directives mapping service names to upstream pools |
| `conf.d/servers/` | Per-service `server {}` blocks (portal, bvr, rts, bvi, public) |
| `conf.d/hardening.conf` | TLS 1.3 only, HSTS, cipher list, OCSP stapling |
| `conf.d/coraza.conf` | Coraza WAF module load and rule-set path |

## How to update

Edit the relevant file and validate before deploying:

```bash
# Validate config syntax
nginx -t -c edge/nginx/nginx.conf

# Build and apply via kustomize (after committing the change)
kubectl kustomize edge/nairobi | kubectl apply -f -

# Or via ArgoCD sync
argocd app sync eel-edge-nairobi
```

Changes to nginx config take effect on pod restart (no live reload in the edge cluster;
a rolling restart is triggered by the ConfigMap hash annotation).

## Config and environment

| Variable / annotation | Purpose |
|-----------------------|---------|
| `NGINX_WORKER_PROCESSES` | Set to `auto` in prod; override in local dev |
| `NGINX_RESOLVER` | Internal CoreDNS address for upstream resolution |
| Config checksum annotation | Triggers rolling restart on ConfigMap change |

## Dependencies

- Envoy Gateway: forwards traffic to nginx on the internal service port
- Varnish: nginx proxies cache-eligible requests to Varnish on `127.0.0.1:6081`
- Coraza WAF: nginx-module build; rules path defined in `conf.d/coraza.conf`
- `edge/waf-rules/`: tuned rule sets loaded by Coraza

## Test and verify

```bash
# Syntax check
docker run --rm -v "$(pwd)/edge/nginx:/etc/nginx:ro" nginx:1.26-alpine nginx -t

# Confirm TLS 1.3 only
echo | openssl s_client -connect 185.x.x.1:443 -tls1_2 2>&1 | grep "Cipher\|alert"
# Expect: "alert handshake failure" (TLS 1.2 rejected)
echo | openssl s_client -connect 185.x.x.1:443 -tls1_3 2>&1 | grep "Cipher"
# Expect: a TLS_AES_* cipher

# HSTS header present
curl -I https://portal.eel.local | grep strict-transport
```

## Related docs

- `edge/varnish/README.md`: caching layer downstream of nginx
- `edge/waf-rules/README.md`: Coraza rule sets loaded by nginx
- `edge/README.md`: edge layer overview

## Limitations

- Config changes require a pod rolling restart; plan for brief disruption during heavy traffic.
- Coraza WAF runs synchronously in the nginx worker; a misconfigured rule can block all traffic.
  Always validate rule changes in staging (`edge/waf-rules/`) before deploying to prod.
