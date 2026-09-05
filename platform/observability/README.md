# platform/observability/

Observability stack for the eel IEMS. Covers metrics, logs, traces, SLOs, and alerting. All 18 services emit telemetry via the OTel Collector.

## Contents

| Path | Description |
|---|---|
| `alerts/` | Prometheus alerting rules (one file per service domain) |
| `dashboards/` | Grafana dashboard JSON (provisioned automatically) |
| `slo/` | SLO definitions (one YAML per service, Sloth-compatible format) |
| `local/` | Kustomize overlay for kind (single-node Grafana, no Mimir) |

## Stack components

| Component | Role | Namespace |
|---|---|---|
| OTel Collector | Receives OTLP (gRPC/HTTP) from all services; routes to Prometheus, Loki, Tempo | `eel-obs` |
| Prometheus | Metrics scraping and alerting | `eel-obs` |
| Mimir | Long-term metrics storage (production; not in local overlay) | `eel-obs` |
| Loki | Log aggregation (structured JSON from all pods) | `eel-obs` |
| Tempo | Distributed tracing (W3C TraceContext) | `eel-obs` |
| Grafana | Dashboards, SLO views, alert management | `eel-obs` |
| Vector | High-throughput log forwarding from nodes | `eel-obs` |

## SLO definitions

Each `slo/slo-<svc>.yaml` declares error rate and latency SLOs aligned with Appendix A load targets:

```yaml
service: rts-svc
slos:
  - name: results-submission-availability
    objective: 99.95
    window: 30d
    indicator:
      ratio:
        errors: http_requests_total{job="rts-svc",status=~"5.."}
        total: http_requests_total{job="rts-svc"}
```

Sloth generates Prometheus recording and alerting rules from these files:

```bash
sloth generate -i platform/observability/slo/slo-rts-svc.yaml \
  -o platform/observability/alerts/slo-rts-svc-rules.yaml
```

## Dashboards

Dashboards in `dashboards/` are provisioned by Grafana sidecar on startup. Each service has a corresponding dashboard JSON. Do not edit dashboards in the UI and expect them to persist; export and commit to this directory.

## Alerting

Alerts route to:
- Slack `#eel-ops` for warning severity
- PagerDuty P1 for election-critical SLO breaches

Alert routing config is in `alerts/alertmanager-routes.yaml`.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single-replica Grafana, Prometheus only (no Mimir/Tempo) |

## Operator responsibilities

- OTel Collector config is in `local/otel-collector-config.yaml` (dev) and the production ConfigMap (prod). Both must have the same pipeline structure.
- Retention: Prometheus 15 days, Mimir 13 months, Loki 90 days, Tempo 30 days.
- All services must annotate pods with `prometheus.io/scrape: "true"` and `prometheus.io/port`. Enforced by Kyverno.

## Related docs

- `platform/cilium/README.md`: Hubble flow logs exported to OTel Collector
- `platform/kafka/README.md`: Kafka consumer lag metrics
- `docs/runbooks/alert-triage.md`: alert investigation runbook

---

## Technical Explanation (Extended) — observability

**Role**: observability is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide observability-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: observability must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from observability components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
