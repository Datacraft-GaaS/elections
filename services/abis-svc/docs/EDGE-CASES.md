# abis-svc Edge Cases

Edge cases from `docs/plans/08-edge-case-matrix.md` that fall within `abis-svc` ownership,
with mapping to concrete tests.

| EC-ID | Category | Edge Case | Concrete Test | Location |
|---|---|---|---|---|
| EC-CON-04 | connectivity | Site partition primary/mirror | `eel-abis-pg-primary-loss.yaml` chaos + outbox-resume assertion | `services/abis-svc/chaos/` |
| EC-CON-05 | connectivity | Flapping link | `vendor_failover_test.rs` retry-budget assertion | `services/abis-svc/tests/` |
| EC-CON-11 | connectivity | Kafka ISR shrink | `enrolment_dedup_flow_test.rs` with simulated Kafka partition | `services/abis-svc/tests/` |
| EC-DAT-01 | data | Duplicate submission, same key+body | `enrolment_dedup_flow_test.rs` idempotency case | `services/abis-svc/tests/` |
| EC-DAT-02 | data | Same key, different body | `crates/abis-api/tests/api_test.rs` 422 assertion | `crates/abis-api/tests/` |
| EC-DAT-07 | data | Partial batch failure | `crates/abis-matcher/tests/matcher_test.rs` partial-batch | `crates/abis-matcher/tests/` |
| EC-IDN-04 | identity | Attestation fails | `attestation_replay_test.rs` MEETS_NONE stub | `services/abis-svc/tests/` |
| EC-IDN-08 | identity | Duress finger | `crates/abis-matcher/tests/matcher_test.rs` duress path | `crates/abis-matcher/tests/` |
| EC-SCL-03 | scale | Hot partition | `enrolment-throughput.js` + Kafka stream-id spread check | `services/abis-svc/loadtest/` |
| EC-SCL-07 | scale | 250k check-ins, 60k RTS, 18k BVI at 1x | Gatling `AbisScenarios.scala` 30-min combined run | `services/abis-svc/loadtest/` |
| EC-SEC-03 | security | HSM unavailable | `attestation_replay_test.rs` degraded-mode assertion | `services/abis-svc/tests/` |
| EC-SEC-04 | security | KEK rotation mid-cycle | OpenBao transit rotate under writes (integration, manual) | `docs/OPERATIONS.md` |

Tags: B = backend surface (all rows above apply to the `abis-svc` backend layer).

Run integration tests:
```
cargo test -p abis-api --test integration_test --features cross-crate-integration
```

See `services/abis-svc/tests/README.md` for full run instructions.
