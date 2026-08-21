# abis-svc Threat Model

STRIDE analysis. References: ADR-0005 (mTLS/SPIFFE), ADR-0016 (data classification).

## Spoofing

Threat: a rogue service submits match requests impersonating the BVR backend and probes the gallery for arbitrary voters.

Control: every inbound call requires a valid SPIFFE SVID presented over mTLS. The API middleware validates the SVID against the eel trust domain and checks caller_id matches the SVID URI SAN. Requests with absent or mismatched SVIDs are rejected at the TLS handshake. Reference: ADR-0005.

## Tampering

Threat: an attacker modifies a match result in transit or on disk to change a `no_match` to `match`.

Controls:
- Match results are written transactionally with an outbox event; the relay signs both with Ed25519 before publishing.
- Model binaries are verified with cosign against the eel model registry before loading into abis-vendor.
- The enclave attestation hash ties each result to the hardware enclave that computed it; downstream consumers can re-verify.

## Repudiation

Threat: an officer denies having submitted or resolved an adjudication case.

Controls:
- Every API mutation writes a row to `eel_abis_audit_shadow` within the same DB transaction as the business mutation.
- The audit shadow rows are replicated to an append-only transparency log (T1). The log is outside the write path of any single operator.
- Adjudication decisions are dual-control: two distinct SpiceDB roles must authorize open and resolve. One actor cannot complete both steps.

## Information Disclosure

Threat: biometric templates or voter IDs leak from the matcher process or the database.

Controls:
- Templates are never stored in the database. `eel_template_index` stores only a probe_ref (URI) and a SHA-256 hash of the encrypted blob. The blob itself lives in the secure store (bio-store), access-controlled separately.
- abis-matcher pods run inside SEV-SNP or TDX enclaves. Memory is encrypted; the hypervisor cannot read plaintext biometric data.
- `probe_ref` and `candidate_ref` are omitted from Kafka event payloads (data class A fields stay inside the service boundary).
- Column-level classification in `data_dictionary.yaml` drives CI checks; any class-A column added to an outbox payload fails the pipeline.

## Denial of Service

Threat: a flood of match requests exhausts DB connections or matcher worker capacity.

Controls:
- The API enforces per-caller rate limits keyed on SVID.
- Kafka provides natural back-pressure; the matcher worker processes at its own pace.
- DB connection pools are bounded per pod; overflow requests return HTTP 429 with a Retry-After header.
- Load shedding is configured at the ingress layer (priority queue: adjudication ops over batch dedup).

## Elevation of Privilege

Threat: a compromised adjudication officer account approves their own duplicate case.

Controls:
- SpiceDB enforces dual-control: the `adjudication_open` permission and `adjudication_resolve` permission are held by different roles.
- The API resolves both permissions at commit time; a single actor holding both is rejected.
- Matcher worker pods run as a non-root UID inside the enclave; no path to escape to the host or access the DB directly.
