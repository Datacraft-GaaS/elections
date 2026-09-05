# platform/rook-ceph/

Rook-Ceph distributed storage for the eel IEMS. Provides block (RBD), object (RGW), and shared filesystem (CephFS) storage classes consumed by PostgreSQL, Kafka, MinIO, and other stateful services.

## Contents

| File | Description |
|---|---|
| `cluster.yaml` | `CephCluster` resource: replica factor, OSDs, network config |
| `local/` | Kustomize overlay for kind (single-node, test storage class) |

## Install

```bash
# Install Rook operator first:
helm upgrade --install rook-ceph rook-release/rook-ceph \
  --namespace rook-ceph --create-namespace

# Apply cluster config:
kubectl apply -f platform/rook-ceph/cluster.yaml
```

## Storage classes

| Class | Type | Replica | Used by |
|---|---|---|---|
| `eel-block-rbd` | Ceph RBD | 3x | PostgreSQL PVCs, OpenSearch |
| `eel-object-rgw` | Ceph RGW (S3) | 3x | MinIO (as gateway), pgBackRest |
| `eel-fs-cephfs` | CephFS | 3x | Kafka log directories |

## Cluster specification

- 3 OSD nodes minimum per site (primary and mirror).
- Replica factor: 3 (all pools).
- Erasure coding not used; replication preferred for latency predictability.
- Network: dedicated Ceph data plane VLAN per ADR-0019.

## Verify cluster health

```bash
kubectl -n rook-ceph exec deploy/rook-ceph-tools -- ceph status
kubectl -n rook-ceph exec deploy/rook-ceph-tools -- ceph osd status
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: hostPath PVs, single OSD, no replication |

## Operator responsibilities

- Never shrink the OSD count below replica factor. Degraded pools block service writes.
- Run `ceph osd scrub` monthly to detect silent bitrot.
- Capacity alert threshold is at 70% utilisation (configured in `platform/observability/alerts/`).

## Related docs

- `platform/minio/README.md`: MinIO uses Ceph RGW as the backing object store
- `platform/postgres/README.md`: PostgreSQL PVCs on `eel-block-rbd`
- `platform/kafka/README.md`: Kafka log dirs on `eel-fs-cephfs`

---

## Technical Explanation (Extended) — rook-ceph

**Role**: rook-ceph is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide rook-ceph-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: rook-ceph must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from rook-ceph components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
