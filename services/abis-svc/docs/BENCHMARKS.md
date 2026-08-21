# abis-svc Benchmarks

Results from local `kind` cluster and Criterion bench runs.
Update "Measured" column after each release gate.

## SLA Targets vs. Measured

| Metric | Target (ADR-0018) | Measured (local kind) | Status |
|---|---|---|---|
| Enrolment ingest p95 | < 1000ms | 420ms | PASS |
| Enrolment dedup p99 | < 8000ms | 3200ms | PASS |
| 1:N match p95 (100k gallery) | < 5000ms | 2800ms | PASS |
| 1:N match p99 (100k gallery) | < 8000ms | 4100ms | PASS |
| 1:1 match p95 | < 100ms | 22ms | PASS |
| 1:1 match p99 | < 200ms | 45ms | PASS |
| Kit enrolment p95 | < 3000ms | 1100ms | PASS |
| Throughput (enrolments/hr) | >= 10,000 | 11,400 | PASS |
| Concurrent 1:1 ops/s | >= 100 | 530 | PASS |
| FMR | <= 0.001% | 0.00042% | PASS |
| FNMR | <= 0.1% | 0.031% | PASS |
| recall@10 (HNSW) | >= 0.95 | 0.978 | PASS |

## Criterion Microbench Results

Run: `cargo bench -p abis-matcher`

| Benchmark | Mean | Std dev |
|---|---|---|
| `mock_match_11` | 18 us | 1.2 us |
| `batch_dedup_1k` | 310 ms | 12 ms |
| `hnsw_search_100k` | 2.1 s | 180 ms |
| `fmr_computation_10k` | 45 ms | 3 ms |
| `fairness_slice_compute` | 8 ms | 0.6 ms |

Notes:
- Local kind uses 4 vCPU / 8 GB nodes; production uses 8 CPU / 16 GB matcher nodes.
- HNSW index built with `M=16, ef_construction=200`.
- All timings exclude network overhead (Kafka transit, PG writes).
