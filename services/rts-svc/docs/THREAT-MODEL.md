# RTS Service Threat Model

## Assets

- Election results (Form 34A, 34B, 34C tallies)
- Form 34A scan images (raw, pre-OCR)
- Cryptographic publication bundles and Merkle roots
- Device SVIDs and signing keys

## Threats and Mitigations

| Threat | Description | Mitigation |
|---|---|---|
| Tampered submission | Attacker modifies vote counts in transit | Ed25519 device signatures on each form; SVID mTLS on all API endpoints |
| Replay attack | Attacker replays a valid form submission with different counts | Idempotency-Key header; stream-level optimistic concurrency in EventStore |
| Insider threat | Privileged operator modifies projection tables directly | Audit shadow triggers (V0003 migration); event log is append-only and hash-chained |
| Double tally | Same station counted twice in 34B compilation | Compiled-stations set tracked per Form34B; EC-DAT-10 arithmetic check |
| Result suppression | Operator withholds a station's form | Transparency log (tlog) records Merkle root; anyone can verify completeness |
| Parallel aggregator drift | Two independent aggregators produce different totals | DriftDetector emits `parallel_tally_drift` event when any candidate total differs |
| OCR manipulation | ICR worker injects incorrect field values | Confidence gate (0.90); requires_review flag routes low-confidence results to human review |
| Kafka message forgery | Attacker injects events onto Kafka topics | SPIFFE mTLS between producers and brokers; exactly-once Kafka transactions |
