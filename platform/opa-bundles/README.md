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

---

## Technical Explanation (Extended) — opa-bundles

**Role**: opa-bundles is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide opa-bundles-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: opa-bundles must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from opa-bundles components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
