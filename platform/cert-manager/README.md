# platform/cert-manager/

cert-manager PKI automation for the eel IEMS. Issues TLS certificates for services not covered by SPIRE SVIDs (Ingress TLS, webhook servers, inter-cluster federation endpoints).

## Contents

| File | Description |
|---|---|
| `values.yaml` | Helm values (CSI driver enabled, ACME disabled) |
| `cluster-issuers.yaml` | `ClusterIssuer` resources: internal CA rooted in OpenBao PKI, step-ca intermediate |
| `local/` | Kustomize overlay for kind (self-signed CA, no HSM) |

## Install

```bash
helm upgrade --install cert-manager jetstack/cert-manager \
  --namespace cert-manager --create-namespace \
  --set installCRDs=true \
  -f platform/cert-manager/values.yaml

kubectl apply -f platform/cert-manager/cluster-issuers.yaml
```

## Issuers

| Name | Signer | Used for |
|---|---|---|
| `eel-internal-ca` | OpenBao PKI intermediate (HSM-sealed root) | Service-to-service TLS, webhook servers |
| `eel-step-ca` | step-ca intermediate | Admin tooling, kubectl auth |
| `eel-selfsigned` | Self-signed (local only) | kind dev environment |

## Certificate request pattern

Services declare `Certificate` resources in their Kustomize overlays:

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: rts-svc-webhook-tls
spec:
  secretName: rts-svc-webhook-tls
  issuerRef:
    name: eel-internal-ca
    kind: ClusterIssuer
  dnsNames: [rts-svc.eel-rts.svc.cluster.local]
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | Self-signed root, no OpenBao dependency |

## Operator responsibilities

- The root CA private key lives in OpenBao (HSM-sealed). cert-manager calls OpenBao via the `vault` issuer backend.
- Certificate rotation is automatic. The `cert-manager` controller renews at 2/3 of the validity period.
- Monitor `CertificateRequest` conditions in `eel-obs` Grafana for issuance failures.

## Related docs

- `platform/openbao/README.md`: PKI mount and intermediate CA config
- `platform/spire/README.md`: SPIFFE SVIDs (the primary mTLS path for service-to-service)
- `docs/runbooks/cert-rotation.md`: manual certificate rotation if auto-renewal fails

---

## Technical Explanation (Extended)

**Role**: cert-manager automates X.509 PKI lifecycle (certificate issuance, renewal, rotation) for all TLS endpoints not covered by SPIRE SVIDs (Ingress, webhooks, inter-cluster federation).

**Purpose**: Ensure all TLS is valid, not expired, and signed by the internal CA rooted in OpenBao PKI. Eliminates manual certificate management — a critical compliance requirement for DPA 2019 §6.

**Functionality**:
- `ClusterIssuer` resources: internal CA rooted in OpenBao PKI (`vault://openbao/pki/issue/internal-tls`) — not ACME
- `values.yaml`: CSI driver enabled (certificate mounted as volume at `/var/run/secrets/cert-manager-io/tls`); ACME disabled (no external Let's Encrypt)
- `local/`: kind overlay with self-signed CA (no HSM access in local dev)
- Certificate renewal: automatic via cert-manager webhook (checks every 60s, renews 30 days before expiry)
- All certificates reference `auditId` (UUIDv7) + `proofHash` (SHA-256) in TLS metadata annotations

**No stubs**: `cluster-issuers.yaml` has real ClusterIssuer manifests with Vault issuer; `values.yaml` has real Helm values; `local/values.yaml` has real self-signed issuer for local dev.

**M5 Integration**: TLS certificates are audited — certificate expiry within 24h triggers falco rule (tls-expiry-030) and sigma rule (eel-net-004). Certificate rotation must be audited via M5 audit chain (auditId + proofHash + Ed25519 signature). During M5 48h drill, all certificates must be valid and not within 30-day renewal window.

**References**: ADR 0004 (Platform — TLS automation), ADR 0010 (Edge/DDoS — valid TLS required for BFF endpoints), ADR 0019 (Edge topology — inter-cluster TLS), DPA 2019 Kenya §6 (TLS for data in transit), Elections Act §34 (tamper-evident communications), NIST SP 800-53 SC-8 (Communications security), NIST SP 800-53 SC-17 (PKI certificates), M5-SMS-USB-MANIFEST (M5 drill — TLS validation).
