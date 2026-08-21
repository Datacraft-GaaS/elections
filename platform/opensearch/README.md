# platform/opensearch/

OpenSearch deployment for the eel IEMS audit log store. Receives structured audit events from all services via the OTel Collector and provides a queryable index for compliance review.

## Contents

| Path | Description |
|---|---|
| `local/` | Kustomize overlay for kind (single-node, no TLS) |

## Index pattern

| Index | Source | Retention |
|---|---|---|
| `eel-audit-*` | Audit events from all services (write, read, admin actions) | 7 years |
| `eel-access-*` | Keycloak access logs, Kong access logs | 2 years |
| `eel-policy-*` | OPA and SpiceDB decision logs | 2 years |
| `eel-ops-*` | Kubernetes audit logs, kubectl actions | 1 year |

Retention is enforced by OpenSearch ISM (Index State Management) policies.

## Install

```bash
helm upgrade --install opensearch opensearch/opensearch \
  --namespace eel-search --create-namespace \
  -f platform/opensearch/local/values.yaml
```

## Query audit log

```bash
# Via kubectl port-forward:
kubectl port-forward -n eel-search svc/opensearch 9200:9200

curl http://localhost:9200/eel-audit-*/_search \
  -H "Content-Type: application/json" \
  -d '{"query": {"match": {"service": "rts-svc"}}}'
```

## Data ingestion

The OTel Collector (`platform/observability/`) routes structured audit log entries to OpenSearch via the `opensearch` exporter. Services emit audit events as structured OTLP log records with `log.type=audit`.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single node, no security plugin, 1 shard per index |

## Operator responsibilities

- 7-year audit retention is a legal requirement (Kenya Data Protection Act). Do not reduce this value.
- Enable OpenSearch Security plugin in production (Basic Auth + TLS). Local overlay disables it for ease of dev.
- Snapshot backups to MinIO via the OpenSearch snapshot repository plugin (daily, retained 90 days).

## Related docs

- `platform/observability/README.md`: OTel Collector routes logs here
- `platform/minio/README.md`: snapshot storage
- `docs/runbooks/audit-log-query.md`: compliance query runbook
