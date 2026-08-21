# bvi-svc Event Catalogue

All events via transactional outbox (ADR-0015). CloudEvents 1.0 envelope.

## Emitted

| type | topic | partitions | retention | signed | delivery |
|---|---|---|---|---|---|
| eel.bvi.session_accepted.v1 | eel.bvi.session.v1 | 16 | operational | yes | at-least-once idempotent |
| eel.bvi.voter_identified.v1 | eel.bvi.identification.v1 | 64 | operational | yes | at-least-once idempotent |
| eel.bvi.turnout_rollup_updated.v1 | eel.bvi.turnout.v1 | 16 | analytics | yes | at-least-once idempotent |
| eel.bve.kit_revoked.v1 | eel.bve.kit.v1 | 16 | operational | yes | exactly-once |
| eel.bvi.distress_received.v1 | eel.bvi.security.v1 | 4 | operational | yes | exactly-once |
| eel.bvi.voter_lookup_audited.v1 | eel.bvi.audit.v1 | 16 | analytics | yes | at-least-once idempotent |
| eel.bvi.objection_filed.v1 | eel.bvi.objection.v1 | 8 | operational | yes | at-least-once idempotent |

## Consumed

| type | topic | role |
|---|---|---|
| eel.bve.kit_attested.v1 | eel.bve.kit.v1 | Kit lifecycle init |
| eel.bvr.voter_status_changed.v1 | eel.bvr.voter.v1 | Register refresh trigger |
| credential_revoked.v1 | eel.bve.kit.v1 | Honey-mode cache update |
