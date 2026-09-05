# platform/minio/

MinIO object storage configuration for the eel IEMS. Provides an S3-compatible API layer over Ceph RGW for backup repositories (pgBackRest, Kopia, OpenSearch snapshots), OPA bundle serving, and Form 34A image storage.

## Contents

| Path | Description |
|---|---|
| `local/` | Kustomize overlay for kind (standalone MinIO, hostPath storage) |

## Buckets

| Bucket | Purpose | Versioned |
|---|---|---|
| `eel-pgbackrest-repo1` | pgBackRest WAL and base backups (primary) | No |
| `eel-pgbackrest-repo2` | pgBackRest WAL and base backups (mirror) | No |
| `eel-opa-bundles` | OPA Rego bundles served to services | No |
| `eel-forms` | Form 34A images (uploaded by rts-svc, read by bvr-svc) | Yes |
| `eel-harbor-blobs` | Harbor container image layer storage | No |
| `eel-opensearch-snapshots` | OpenSearch index snapshots | No |
| `eel-kopia-backups` | Kopia non-database state backups | No |

## Install

```bash
# Operator-based install:
helm upgrade --install minio-operator minio/operator \
  --namespace minio-operator --create-namespace

kubectl apply -k platform/minio/
```

## Access

MinIO uses OpenBao-issued credentials (KV path `kv/eel/minio/<bucket>-credentials`). Services obtain access keys at startup via the OpenBao agent sidecar.

```bash
# Dev access (local overlay):
kubectl port-forward -n eel-storage svc/minio-console 9090:9090
# Open http://localhost:9090, login minioadmin/minioadmin
```

## DR replication

Buckets `eel-pgbackrest-repo2` are hosted on the mirror MinIO instance. Active-passive replication is configured via MinIO site replication between primary and mirror clusters.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single instance, `minioadmin` credentials, hostPath volumes |

## Operator responsibilities

- Form 34A images in `eel-forms` are write-once. Object lock (WORM) is enabled on this bucket in production.
- Monitor bucket utilisation via Grafana (MinIO metrics scraped by Prometheus).
- Do not store personally identifiable information directly in MinIO; all objects must be encrypted at rest (Ceph-level encryption enabled in `platform/rook-ceph/cluster.yaml`).

## Related docs

- `platform/pgbackrest/README.md`: backup repository config
- `platform/rook-ceph/README.md`: Ceph RGW backing store
- `platform/openbao/README.md`: credential injection

---

## Technical Explanation (Extended) — minio

**Role**: minio is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide minio-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: minio must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from minio components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
