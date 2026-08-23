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

The test sources in this directory are wired as cargo test targets by the
`abis-integration-tests` harness crate (`crates/abis-integration-tests`,
WP-3.6); this directory has no manifest of its own.

### Basic (no Docker required)

Tests without `#[cfg_attr(not(feature = "cross-crate-integration"), ignore)]`
run without infrastructure:

```
cargo test -p abis-integration-tests
```


### Full integration (requires Docker + running abis-api)

Start a local `abis-api` first (see `docs/README.md`), then:

```
cargo test -p abis-integration-tests --features cross-crate-integration -- --include-ignored
```

### Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `ABIS_TEST_URL` | `http://localhost:8080` | Base URL of running abis-api |

## Notes

- Tests requiring a live stack stay ignored without the feature and fail loudly on connection errors when the feature is enabled without one.
- Attestation tests use `dev-attest-placeholder` token valid only when `EEL_ABIS_ATTESTATION=software`.
