# Reproducibility bundle (M14)

Module M14 (`reproducibility-bundle`) owned by `transparency-svc`. Builds a verifiable bundle of election events, register snapshots, code hashes, and manifests; implements `make reproduce v2` for deterministic rebuild; issues participation tokens; accepts bug-bounty intake; publishes to Gazette/KBC via SFTP.

## Contents

1. What M14 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M14 does

M14 assembles a post-election reproducibility bundle: a signed, content-addressed archive containing all election events from the transparency log (P05), register snapshots, the exact code commit hashes that ran on election day, and a manifest that lets any independent party reproduce the official result from first principles. The bundle is published to the Kenya Gazette and KBC (Kenya Broadcasting Corporation) via SFTP, issued as participation tokens for independent verifiers, and accepts structured bug-bounty submissions.

## 2. Data plane

Crates: `services/transparency-svc/crates/repro-bundle/` (bundle.rs, manifest.rs, token.rs, error.rs).

| Aggregate | Purpose | Data class |
|---|---|---|
| **ReproBundle** | bundle_id (UUID7), election_id, event_range (start_seq, end_seq), snapshot_ids[], code_hashes (map service -> git_sha), manifest_hash (SHA-256), bundle_sig (Ed25519 + HSM), published_at | A |
| **BundleManifest** | manifest_id, bundle_id, tlog_root, register_root, code_map, reproduction_instructions, schema_version | A |
| **ParticipationToken** | token_id, bundle_id, holder_id_hash, issued_at, scope (VERIFY_RESULT/REPRODUCE/BUG_BOUNTY) | B |
| **BugBountySubmission** | submission_id, token_id, finding_hash, description_encrypted, severity, status (OPEN/TRIAGED/FIXED/CLOSED) | B |
| **PublicationRecord** | pub_id, bundle_id, channel (GAZETTE_SFTP/KBC_SFTP), sftp_path, published_at, receipt_hash | C |

## 3. Ingest sources

| Source | Notes |
|---|---|
| Transparency log (P05) | `services/transparency-svc/crates/tlog-admin/src/export.rs`; export event range by seq |
| M10 rts-backend | TallySnapshot Merkle roots |
| M02 bvr-backend | RegisterSnapshot Merkle roots |
| CI/CD build system | Git commit SHAs per service, build reproducibility attestation (SLSA provenance) |
| Gov Printer / Kenya Gazette | SFTP target; `integrations/gov-printer/` |
| KBC (Kenya Broadcasting Corporation) | SFTP target; bundle tarball |

## 4. Pipeline

Crates: `services/transparency-svc/crates/repro-bundle/src/bundle.rs`, `manifest.rs`, `token.rs`.

```
post-election trigger (manual Commissioner action or scheduled)
      -> tlog-admin/src/export.rs: export all events (start_seq..end_seq)
             from P05 tlog to local bundle directory
      -> collect RegisterSnapshot Merkle roots (M02 API)
      -> collect TallySnapshot Merkle roots (M10 API)
      -> collect code hashes: git log --format=%H per service tag
             SLSA provenance attestation included per service
      -> assemble BundleManifest:
             tlog_root = STH tree_head_hash at end_seq
             register_root = RegisterSnapshot.root_hash
             code_map = { service: sha } for all 17 modules
             reproduction_instructions = make reproduce v2 steps
      -> content-address bundle (SHA-256 of full tarball)
      -> sign bundle: Ed25519 (transparency-svc key) + HSM countersign (P06)
             -> ReproBundle.bundle_sig
      -> publish to channels:
             GAZETTE_SFTP: Gov Printer SFTP (integrations/gov-printer/)
             KBC_SFTP: KBC SFTP endpoint
             PublicationRecord created per channel with receipt_hash
      -> issue ParticipationTokens:
             for each accredited independent verifier: signed JWT scoped to bundle_id
             token.rs: Ed25519 signed, 90-day TTL
      -> accept BugBountySubmission via API:
             encrypted finding; severity triage; token required
      -> emit eel.transparency.repro_bundle_published.v1
```

`make reproduce v2` workflow (in bundle): any party with the bundle tarball runs `make reproduce` to:
1. Replay all tlog events.
2. Recompute tally from Form 34A events.
3. Verify all HSM signatures.
4. Confirm tlog_root matches the published STH.

## 5. API surface

Contract: `packages/openapi-contracts/transparency/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /transparency/v1/repro-bundle` | Commissioner (JWT + HSM-gated) | Trigger bundle assembly |
| `GET /transparency/v1/repro-bundle/{id}` | Public | Download signed bundle manifest |
| `GET /transparency/v1/repro-bundle/{id}/download` | Public | Download signed bundle tarball |
| `POST /transparency/v1/repro-bundle/{id}/tokens` | Commission admin (JWT + SpiceDB) | Issue participation tokens |
| `POST /transparency/v1/bug-bounty` | Token holder (signed JWT) | Submit bug-bounty finding |
| `GET /transparency/v1/bug-bounty/{id}` | Commission admin (JWT + SpiceDB) | Review bug-bounty submission |
| `GET /transparency/v1/tlog/leaves` | Public | Browse tlog leaves |
| `GET /transparency/v1/tlog/proof/{leaf_id}` | Public | Merkle inclusion proof for leaf |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Commissioner bundle panel (Django) | `services/transparency-svc/django/` | Trigger bundle, issue tokens, triage bug bounty |
| Public download (Astro portal) | `services/transparency-svc/astro/` | Public bundle download + make reproduce instructions |
| IAB bundle review | IAB dashboard (M15) via tlog-witness | Independent audit of bundle completeness |

## 7. Integrity + anti-tamper

- **Content-addressed bundle** -- SHA-256 of tarball included in manifest; any bit flip detectable.
- **Ed25519 + HSM countersignature** on every ReproBundle; signature verifiable without private key access.
- **SLSA provenance** -- build provenance attestation per service; verifier can confirm the code hash corresponds to a reproducible build.
- **Transparency log STH** -- tlog_root is the signed tree head at end_seq; any omission of events would change the root.
- **Participation token scope** -- tokens scoped to specific bundle_id and capability (VERIFY_RESULT / REPRODUCE / BUG_BOUNTY); no cross-bundle reuse.
- **Bug-bounty encryption** -- finding description encrypted with Commission public key before storage; only triagers can decrypt.
- **Publication receipt hash** -- SFTP upload receipt hash stored in PublicationRecord; confirms Gov Printer and KBC received the exact tarball.
- **make reproduce v2 determinism** -- all steps in the bundle Makefile are deterministic; any non-determinism in the build chain is a finding.

## 8. Kenya statute alignment

- **Elections Act s.82 (court access to results)** -- reproducibility bundle provides the complete evidentiary package for any election petition.
- **Elections Act s.87 (transparency)** -- bundle publication to Gazette satisfies the statutory transparency requirement for electronic systems.
- **Elections (Technology) Regulations 2017** -- KEBS certifier can use the bundle to independently verify the system's operation.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- bundle contains no voter PII; only hashed IDs, Merkle roots, and aggregated tallies.

## 9. Honest caveats + external prerequisites

- **Gov Printer SFTP endpoint** and **KBC SFTP endpoint** are external prerequisites; adapters written; credentials and access provisioning pending.
- **SLSA provenance** for reproducible builds requires CI/CD pipeline configuration (Nix flakes referenced in `flake.nix`); reproducibility of Rust crates depends on pinned toolchain versions.
- **make reproduce v2** has not been end-to-end tested against a full 30M-voter, 46k-station election event set; a pre-election rehearsal drill is required.
- **Bug-bounty program** scope, rewards, and legal terms are outside this repo and require IEBC legal sign-off before launch.
- **Participation token distribution** mechanism (who qualifies as "accredited independent verifier") is an IEBC policy decision.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (bundle assembly, manifest hash, token signing, bug-bounty encryption) | nextest | M14-UNIT- |
| Property (bundle hash deterministic across two runs; token not reusable across bundles) | proptest | M14-PROP- |
| Integration (tlog export double; M10 / M02 snapshot double; HSM SoftHSM; SFTP mock) | nextest + testcontainers | M14-INT- |
| E2E (make reproduce v2 replay on test election corpus) | make + nextest | M14-E2E- |
| Security (bundle tampering detection; token scope enforcement; bug-bounty decryption access) | Manual + ZAP | M14-SEC- |

Runbook (to create): `docs/runbooks/transparency-svc-repro-bundle.md` (SFTP failure; HSM unavailable at bundle time; make reproduce failure).

Full spec: `docs/plans/modules/reproducibility-bundle.md`. Crates: `services/transparency-svc/crates/repro-bundle/`, `tlog-admin/`, `tlog-api/`, `hsm-signer/`. Astro: `services/transparency-svc/astro/`.
