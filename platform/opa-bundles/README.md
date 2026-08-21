# platform/opa-bundles/

OPA (Open Policy Agent) Rego bundles for the eel IEMS. Provides domain policy evaluation decoupled from individual services.

## Bundle layout

| Directory | Purpose |
|---|---|
| `admission/` | Rego policies called by the OPA admission sidecar (additional layer over Kyverno) |
| `authz/` | Domain authorization policies for services that call OPA for fine-grained decisions not expressible in SpiceDB |

## Key policies

- `authz/results-certification.rego`: validates that a Form 34A submission has the required officer signatures before it is accepted by rts-svc.
- `authz/diaspora-eligibility.rego`: checks voter register cross-reference for vprs-svc.
- `admission/resource-class.rego`: enforces `x-eel-data-class` labels on Secrets and ConfigMaps.

## Bundle build and push

```bash
# Build bundles:
opa build -b platform/opa-bundles/authz/ -o opa-authz.tar.gz
opa build -b platform/opa-bundles/admission/ -o opa-admission.tar.gz

# Push to MinIO (bundle server):
mc cp opa-authz.tar.gz minio/eel-opa/bundles/authz/latest.tar.gz
mc cp opa-admission.tar.gz minio/eel-opa/bundles/admission/latest.tar.gz
```

Services pull bundles from MinIO on startup and on a 60-second poll interval.

## Test

```bash
# Unit tests for all bundles:
opa test platform/opa-bundles/

# Coverage:
opa test --coverage platform/opa-bundles/authz/
```

## Operator responsibilities

- Bundle versions are immutable; always push to a new version path and update the service config to point to it.
- OPA decisions are logged to OpenSearch for audit.
- Rego policies must have `>= 90%` branch coverage before promotion to production.

## Related docs

- `platform/spicedb/README.md`: SpiceDB for relationship-based authz (the primary authz path)
- `platform/openbao/README.md`: secrets used by OPA policies
- `platform/minio/README.md`: bundle server storage
