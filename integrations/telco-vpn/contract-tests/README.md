# telco-vpn/contract-tests

Contract tests for VPN tunnel health monitoring.

## Purpose

Verifies tunnel status reporting, health probe behavior, failover logic, and rekeying notifications.

## Running tests

```bash
cargo test -p eel-contract-tests-telco-vpn
```

Note: may require --features integration-tests and elevated privileges (root) for full suite.

## What is verified

- TunnelStatus schema (name, state enum, packets_sent, bytes_sent)
- Health probe latency (sub-second local call)
- State transitions: up -> rekeying -> up (no loss of service)
- Failover: if primary down, queries secondary
- Dual-tunnel redundancy: either tunnel up = service healthy
- Rekey events reported (no impact to active connections)
- Prometheus metrics accurate (packets_sent, bytes_sent, tunnel_up gauge)

## Environment setup for integration tests

1. Configure strongSwan IPsec tunnel to test peer
2. Configure WireGuard interface wg0
3. Run with --features integration-tests

Standalone unit tests use mock strongSwan CLI responses.
