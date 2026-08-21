# platform/

Deployment configuration for the platform layer per ADR-0004. Everything here is GitOps-managed via ArgoCD.

| directory | contents |
|---|---|
| kubernetes/ | cluster manifests, kubeadm configs, node pools |
| argocd/ | ArgoCD install + ApplicationSets fanning per (service × site) |
| cilium/ | Cilium CNI + mesh + NetworkPolicy + Hubble config |
| envoy-gateway/ | Envoy Gateway install + routes |
| karapace/ | schema registry (Aiven fork of Confluent SR) |
| keycloak/ | realms as JSON, brokered IdPs, WebAuthn policy, MFA rules |
| openbao/ | mounts, PKI intermediates, HSM seal config, dynamic secret roles |
| spicedb/ | deployment, backup, zookie propagation policy |
| spire/ | server + agent, attestor selectors (K8s SA + TPM + HSM) |
| kafka/ | Kraft cluster, tiered storage, topic definitions (one YAML per topic) |
| debezium/ | connectors per T1 cluster (outbox → Kafka) |
| harbor/ | container registry with cosign verification enforced |
| rook-ceph/ | block/object/file storage |
| observability/ | OTel Collector + Prometheus + Loki + Tempo + Grafana + Mimir + Vector |
| cert-manager/ | issuers rooted in step-ca + HSM |
| kyverno-policies/ | admission policies (cosign verify, mesh membership, resource limits) |
| opa-bundles/ | Rego bundles for domain policy |
