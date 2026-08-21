# abis-svc Integration Tests

Cross-crate integration tests spanning `abis-api`, `abis-matcher`, and `abis-attest`.

## Structure

```
tests/
  common/mod.rs                   shared test rig helpers
  integration_test.rs             placeholder (existing)
  enrolment_dedup_flow_test.rs    full pipeline: ingest -> quality -> dedup -> route -> enrol
  adjudication_workflow_test.rs   case lifecycle: created -> assigned -> decided -> tlog
  fairness_drill_test.rs          cohort fairness breach and pass scenarios
  attestation_replay_test.rs      nonce replay and stale nonce rejection
  vendor_failover_test.rs         vendor router failover and circuit-breaker
```

## Running

### Basic (no Docker required)

Tests without `#[cfg_attr(not(feature = "cross-crate-integration"), ignore)]` run without infrastructure:

```
cargo test --test attestation_replay_test
cargo test --test vendor_failover_test
```

### Full integration (requires Docker + running abis-api)

Start a local `abis-api` first (see `docs/README.md`), then:

```
cargo test --test enrolment_dedup_flow_test --features cross-crate-integration
cargo test --test adjudication_workflow_test --features cross-crate-integration
cargo test --test fairness_drill_test --features cross-crate-integration
cargo test --test attestation_replay_test --features cross-crate-integration
cargo test --test vendor_failover_test --features cross-crate-integration
```

Or run all at once:

```
cargo test --features cross-crate-integration -- --include-ignored
```

### Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `ABIS_TEST_URL` | `http://localhost:8080` | Base URL of running abis-api |

## Notes

- Tests requiring `abis-enrolment` or `abis-adjudication` crates are gated behind `cross-crate-integration` feature and will be skipped (`#[ignore]`) until those crates exist.
- Tests print `SKIP:` and exit 0 when Docker is unavailable (no testcontainers).
- Attestation tests use `dev-attest-placeholder` token valid only when `EEL_ABIS_ATTESTATION=software`.
