# abis-svc Events

All events are CloudEvents v1.0, serialized as JSON, published via the transactional outbox (V007__outbox.sql). The outbox relay signs payloads with Ed25519 before forwarding to Kafka.

## match_requested.v1

Emitted when a 1:N match is submitted to the API.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.match.v1` |
| Data class | C |
| Retention | 90 days |
| Signed | Yes (Ed25519) |
| Delivery | At-least-once via outbox relay |

Payload fields: `request_id`, `probe_ref`, `modality`, `election_id`, `caller_id`, `submitted_at`.

## match_completed.v1

Emitted when the matcher worker finishes scoring and writes a result row.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.match.v1` |
| Data class | C |
| Retention | 90 days |
| Signed | Yes (Ed25519) |
| Delivery | At-least-once via outbox relay |

Payload fields: `request_id`, `result_id`, `outcome`, `confidence_score`, `modality_scores`, `attestation_hash`, `completed_at`.

`attestation_hash` links the result to the enclave that produced it. Consumers verifying audit trails MUST check this field is present and non-empty.

## adjudication_opened.v1

Emitted when an ambiguous match score (between lower and upper thresholds) creates an adjudication case.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.adjudication.v1` |
| Data class | B |
| Retention | 7 years (election records) |
| Signed | Yes (Ed25519) |
| Delivery | At-least-once via outbox relay |

Payload fields: `case_id`, `match_result_id`, `match_score`, `sla_deadline_at`, `created_at`.

probe_ref and candidate_ref are NOT included in the event payload (data class A; not published to broker). Consumers needing them must call the adjudication API with appropriate SpiceDB permissions.

## adjudication_decided.v1

Emitted when an officer resolves an adjudication case.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.adjudication.v1` |
| Data class | B |
| Retention | 7 years |
| Signed | Yes (Ed25519) |
| Delivery | At-least-once via outbox relay |

Payload fields: `case_id`, `decision`, `reason_code`, `investigator_id`, `resolved_at`.

## enclave_attested.v1

Emitted after the abis-attest crate successfully verifies a worker's attestation quote.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.enclave.v1` |
| Data class | C |
| Retention | 90 days |
| Signed | Yes (Ed25519) |
| Delivery | At-least-once via outbox relay |

Payload fields: `enclave_id`, `worker_id`, `profile`, `attestation_hash`, `verified_at`.

## drift_alert.v1

Emitted when the FMR or FNMR metric (or a per-group fairness metric) crosses the configured tolerance band.

| Field | Value |
|-------|-------|
| Topic | `eel.abis.drift.v1` |
| Data class | D |
| Retention | 30 days |
| Signed | No (aggregate metrics, no PII) |
| Delivery | At-least-once via outbox relay |

Payload fields: `alert_id`, `drift_type`, `value`, `threshold`, `modality`, `demographic_group`, `model_card_version`, `detected_at`.

Consumers: model-ops dashboard, on-call alerting pipeline.
