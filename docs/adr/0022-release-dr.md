# ADR 0022: Release process & Disaster Recovery

Status: accepted
Date: 2026-08-21

## Context

`sw/ARCHITECTURE.md` §14.3 mandates blue-green with 15-minute hot rollback for stateful services and RTO ≤30min / RPO ≤5min for the results transmission service (`rts-svc`). Combined with the RFP-binding 30-day pre-election software freeze (ADR-0014) and two-site Commission-provided DR mandate, this deserves its own ADR rather than being scattered across 0004/0014/0018.

## Decision

### Release model

- **GitOps** via ArgoCD (ADR-0004). Every environment is one directory of manifests.
- **Environments** (four):
  - `dev`, per-team ephemeral namespaces, no PII, synthetic voter register (ADR-0016).
  - `staging`, full production topology in shape (2 sites + 5 edges), reduced capacity, synthetic data.
  - `pre-prod`, full production topology at full capacity, synthetic data, load-test target, chaos-engineering target.
  - `prod`, the two the Commission DR sites + five regional edges (ADR-0019).
- **Release train**:
  - Nightly deploy to `dev`.
  - Weekly deploy to `staging` on Wednesdays; must pass full CI + Pact contract tests + Semgrep/Bandit + Trivy/Grype.
  - Bi-weekly deploy to `pre-prod` on green-only. Full load test (5x peak, ADR-0018) runs against pre-prod every deploy from month 2 onwards.
  - Manual deploy to `prod`, quorum approval (2 humans + ops on-call), tabletop-reviewed release notes.
- **Deployment strategy**: **Blue-green** for stateful services (rts-svc, bvr-svc, bvi-svc, abis-svc, transparency-svc, ecc-svc). **Rolling** for stateless services. **Canary (10% → 50% → 100%)** for public-portal services (portal-svc, vprs-svc) during non-freeze windows.
- **Hot rollback**: 15 minutes maximum from decision to rollback complete for any service. Achieved by keeping blue+green both warm and DB migrations rollback-tested (ADR-0014).

### DR posture

- **Two the Commission DR sites** (primary + mirror, per RFP). Primary in Anniversary Towers (or Commission-designated), mirror in Bomas of Kenya (or Commission-designated).
- **Data replication**:
  - T1 clusters: **synchronous streaming replication** primary → mirror (Patroni + `synchronous_commit = remote_write`).
  - T2 clusters: **asynchronous logical replication**.
  - Kafka event-source topics: **MirrorMaker 2** primary → mirror (asynchronous, RPO < 10s).
  - Object storage: MinIO site-to-site replication (async).
- **SLOs**:
  - `rts-svc` write path: **RTO ≤ 30 minutes, RPO ≤ 5 minutes** (spec-binding).
  - Other T1 services: RTO ≤ 60 min, RPO ≤ 15 min.
  - T2 services: RTO ≤ 4 hours, RPO ≤ 1 hour.
  - Edge clusters: RTO ≤ 15 min per region (DNS anycast failover to nearest healthy region within seconds; full service capacity restore within 15 min).
- **Failover procedure**:
  - Documented per service in `docs/runbooks/dr-failover-<service>.md`.
  - **Rehearsed monthly** in staging; **quarterly in pre-prod at full scale**.
  - Automated for edge cluster loss; manual (with automation-assist) for site failover.
- **Backup discipline**:
  - Postgres: **pgBackRest** with WAL streaming to MinIO. Full weekly, differential daily, WAL continuous. **PITR to any point in 30 days**.
  - Kafka: Tiered Storage to MinIO (ADR-0015 amendment: 90d hot per RFP alignment).
  - Cluster state: **Velero** to MinIO on the opposite site.
  - Application data-at-rest: **Kopia** to MinIO with per-tier encryption keys (ADR-0016).
  - **Restore drills monthly**: cluster full restore to a scratch environment; any drill failure blocks the next release.
- **Third defence-in-depth site (optional, not RFP-required)**: reproducibility-bundle publication mirror only; not a full replica. Decision deferred to budget.

### Election-day mode

- **T-30 days**: schema freeze (ADR-0014), release freeze except CVE-critical, on-call ramp to 24/7.
- **T-7 days**: full chaos-engineering drill + full DR failover rehearsal at pre-prod.
- **T-24h to T+72h**: two-humans-in-room for any prod action; comms cadence pre-committed to hourly public updates; NOC + SOC fully staffed.
- **T-0 (election day)**: no deploys of any kind; only feature-flag flips authorised and only by dual sign-off.

### Regression gate

- No release may regress P99 write latency by >10% at 5x load (ADR-0018).
- No release may regress error rate by >0.1 percentage points at 1x load.
- No release ships if the last DR restore drill failed.
- Any failed regression = automatic rollback to previous version within 15 minutes.

## Consequences

Positive: SLO numbers are concrete and rehearsed; blue-green + 15-min rollback bounds blast radius on any single bad release; election-day mode is codified, not improvised.

Negative: blue-green doubles the resource requirement for stateful services during release window (accepted). Monthly rehearsal cost is real; non-negotiable.

## Alternatives rejected

- Rolling deploys only: cannot achieve 15-minute rollback with DB migrations in flight.
- Manual DR: RTO ≤ 30 min impossible without automation-assist.
- Three-site synchronous replication: RFP requires only two; third is defence-in-depth if funded, not baseline.

## Related

ADR-0004 (platform), ADR-0012 (T1 topology), ADR-0014 (schema freeze), ADR-0015 (Kafka MirrorMaker 2), ADR-0018 (write path SLA), ADR-0019 (edge topology).
