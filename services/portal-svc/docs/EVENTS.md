# portal-svc event contracts

## Consumed events

| Event type | Topic | Delivery | Notes |
|---|---|---|---|
| eel.rts.snapshot_published.v1 | eel.rts.snapshot.v1 | exactly-once | Drives snapshot_index upsert and cache invalidation |
| eel.bvi.turnout_rollup_updated.v1 | eel.bvi.turnout.v1 | at-least-once idempotent | Live turnout feed; 60 s configurable delay |

## Emitted events

| Event type | Topic | Delivery | Notes |
|---|---|---|---|
| eel.portal.snapshot_served.v1 | eel.portal.audit.v1 | at-least-once idempotent | Audit log of snapshot serves; aggregate counts only, no PII |

## PubSub topics (internal)

| Topic | Publisher | Subscriber | Notes |
|---|---|---|---|
| eel:counts | Kafka consumer projection | CountsLive, SseController | Strictly aggregate; no PII |
