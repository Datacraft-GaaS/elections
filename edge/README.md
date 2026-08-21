# edge/

Five regional Kubernetes edge clusters per ADR-0019 plus edge-layer WAF, rate limiting, and origin shield config.

| region | city | serves |
|---|---|---|
| nairobi/ | Nairobi | Central + Rift Valley South |
| mombasa/ | Mombasa | Coast |
| kisumu/ | Kisumu | Nyanza + Western |
| eldoret/ | Eldoret | Rift Valley North + Turkana + West Pokot + Trans-Nzoia |
| garissa/ | Garissa | North East |

- **waf-rules/** — Coraza WAF rules per service, tuned not default CRS
- **rate-limits/** — Envoy Gateway sliding-window limits per (IP, token, API-key, endpoint)
- **nginx/** — origin-shield config
- **varnish/** — long-TTL signed-snapshot caching
- **anubis/** — proof-of-work bot mitigation, escalation ladder
