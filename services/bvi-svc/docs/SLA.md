# bvi-svc SLA

Availability class: election-critical 99.99% during polling window.

| Metric | Target | Alert |
|---|---|---|
| Ingest p99 | <= 100 ms at 18,000 req/s | > 100 ms for 1 min |
| Event-to-dashboard latency | <= 30 s | > 30 s for 2 min |
| Kit revocation propagation | <= 60 s | > 60 s for any kit |
| BVV lookup p99 | <= 2 s | > 2 s for 5 min |
| Availability (polling window) | 99.99% | Burn rate > 1x for 1 h |
| RTO | <= 30 min | |
| RPO | <= 5 min | |

BVV (M07): routine 99.5% outside polling window.
