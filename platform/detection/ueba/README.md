# UEBA: User and Entity Behaviour Analytics

UEBA provides continuous behavioural profiling of all principals in the eel election platform. It ingests Falco and Tetragon events from Loki, learns per-entity baselines over a 14-day rolling window, and raises alerts via PostgreSQL pubsub to the ECC (Election Control Centre) dashboard when observed behaviour deviates significantly from the learned norm.

## Entity Types

| Type | Description |
|---|---|
| `user` | Human operators authenticated via Keycloak |
| `service_account` | Kubernetes service accounts used by platform services |
| `kit` | Physical election kit devices (tablets, scanners) |
| `pod` | Individual Kubernetes workload pods |

## Features Tracked

| Feature | Description |
|---|---|
| `login_hour_of_day` | Hour (0-23) at which the entity authenticates |
| `query_rate_per_minute` | Database query throughput |
| `syscall_entropy` | Shannon entropy of system call mix observed by Tetragon |
| `bytes_egressed` | Network bytes sent outbound |
| `distinct_tables_accessed` | Cardinality of DB tables touched per session |

## Architecture

```
Falco / Tetragon
      |
      v
   Loki (log aggregation)
      |
      v
 baseline-learner.py          anomaly-detector.py
  (writes eel_ueba_baselines)  (reads baselines, z-scores live stream)
      |                              |
      +---------- asyncpg -----------+
                       |
              NOTIFY eel.ecc.ueba_alert.v1
                       |
                  ECC Dashboard (listener)
```

## Data Flow

1. Falco rules (in `rules/`) emit structured JSON log lines to stdout.
2. Tetragon TracingPolicies emit kernel-level event streams.
3. Both are scraped by the Loki agent on each node with labels `{job="falco"}` and `{job="tetragon"}`.
4. `baseline-learner.py` queries Loki every `query_interval_seconds` (default 30 s) using the LogQL HTTP API. For each entity seen in the interval it aggregates feature vectors and updates `eel_ueba_baselines` in PostgreSQL with the rolling mean and standard deviation over the last 14 days.
5. `anomaly-detector.py` tails the Loki stream in real time. For each incoming event it fetches the stored baseline for the relevant entity and computes per-feature z-scores: `z = (value - mean) / std`. Any feature with `z > 3.0` (configurable via `zscore_threshold`) triggers an alert.
6. Alerts are published via PostgreSQL `NOTIFY eel.ecc.ueba_alert.v1` with a JSON payload. The ECC dashboard maintains a persistent `LISTEN` connection and surfaces alerts in real time.

## Alert Severity

| Condition | Severity |
|---|---|
| Any feature z-score > 3.0 | WARNING |
| Any feature z-score > 5.0 | CRITICAL |

A 300-second cooldown per entity prevents alert storms.

## Baseline Schema

Table `eel_ueba_baselines`:

```sql
CREATE TABLE eel_ueba_baselines (
    entity_id        TEXT NOT NULL,
    entity_type      TEXT NOT NULL,
    feature          TEXT NOT NULL,
    mean             DOUBLE PRECISION NOT NULL,
    std              DOUBLE PRECISION NOT NULL,
    sample_count     INTEGER NOT NULL,
    window_start     TIMESTAMPTZ NOT NULL,
    window_end       TIMESTAMPTZ NOT NULL,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (entity_id, entity_type, feature)
);
```

## Configuration

See `ueba-config.yaml`. All secrets (DSN, Loki URL) are injected via environment variables and Kubernetes secrets.

## Deployment

See `deploy/deployment.yaml`. The worker runs as 2 replicas under the `eel-ueba-sa` ServiceAccount with least-privilege RBAC.

## Rules

| File | Engine | Description |
|---|---|---|
| `rules/after-hours-access.yaml` | Falco | Logins or API calls between 22:00 and 06:00 EAT |
| `rules/unusual-query-pattern.yaml` | Falco | Query rate > 3 sigma above baseline for service accounts |
| `rules/syscall-drift.yaml` | Tetragon | Syscalls outside the learned pod profile |
| `rules/kit-behaviour-drift.yaml` | Falco | Kit egress bytes > 3 sigma above baseline |
| `rules/mass-download-alert.yaml` | Falco | More than 10,000 records fetched in a 5-minute window |
