# governance-svc runbook (WP-5.1)

## Deploy

```bash
kubectl apply -f deploy/service.yaml
kubectl rollout status deployment/governance-svc
```

## RLA audit run

1. Upload ballot manifest via POST /api/v1/governance/rla/audits
2. Fetch NIST beacon seed, derive IAB card seed
3. Draw sample via POST .../{run_id}/samples
4. Load sample on rla-desktop (offline audit laptop)
5. Record verdicts; desktop queues for sync
6. On reconnect, sync fires and verdicts upload
7. GET .../{run_id}/report -- if recount_triggered, escalate

## Court evidence bundle

POST /api/v1/governance/court/petitions -- file petition
POST .../{petition_id}/evidence-bundles -- attach 15-item bundle
GET .../{petition_id}/evidence-bundles/{bundle_id} -- download for court

## Health checks

GET /health on each port (8091-8095).
