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
