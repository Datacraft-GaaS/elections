# RTS Service SLA Targets (ADR-0018)

| Metric | Target | Notes |
|---|---|---|
| Ingest p99 latency (local) | <= 2s | Measured at `POST /api/v1/rts/forms` under 600 VUs (1/100 scale) |
| Ingest p99 latency (satellite uplink) | <= 8s | Includes uplink delay budget |
| Sustained write throughput | 50,000 writes/sec | Across primary + mirror sites |
| Form 34A capture to 34B compilation | <= 10 min | After all stations in a constituency have verified |
| National 34C compilation | <= 30 min | After all constituencies compiled |
| Publication (tally_published event) | <= 60 min | After 34C compiled |
| Availability | 99.9% | Per election period; active-active primary + mirror |
| Recovery time objective | 15 min | Hot standby DB failover |
| Recovery point objective | 0 events | Kafka exactly-once; no events lost on failover |

## Load Test

Run `loadtest/k6/rts-submit.js` with `RTS_BASE_URL` pointing at a staging instance:

```sh
k6 run --env RTS_BASE_URL=http://staging-rts:8080 loadtest/k6/rts-submit.js
```

The script targets 600 VUs for 5 minutes and asserts p99 < 2000ms.
