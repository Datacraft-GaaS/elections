# eel-governance-oag

M59 Auditor-General interface: IFMIS evidence export, audit engagement tracking, evidence scope definitions.

## What it does

Manages election audit engagement with the Office of the Auditor-General (OAG). Tracks audit engagements, defines evidence scope (categories: procurement, payroll, asset disposal), exports evidence packages to IFMIS via the standardized audit evidence exchange format, and maintains evidence chain-of-custody logs. Integrates with support-admin-svc (payroll, budget) and transparency-svc (evidence archival).

Fits in governance-svc as the OAG liaison layer. Runs on port 8095; exports evidence packages on schedule (weekly + on-demand); integrates with OpenBao for OAG mTLS client certificate provisioning.

## Public API

`eel_governance_oag::engagement::AuditEngagement` aggregate: `fn define_evidence_scope(&mut self, scope: EvidenceScope) -> Result<()>`, `fn export_package(&self, format: ExportFormat) -> Result<EvidencePackage>`, `fn sign_handover(&mut self, signature: Signature) -> Result<()>`.

Endpoints:
- `POST /v1/governance/oag/audit-engagements` - initiate engagement
- `GET /v1/governance/oag/audit-engagements/{id}` - fetch metadata
- `POST /v1/governance/oag/audit-engagements/{id}/evidence-scope` - define categories
- `POST /v1/governance/oag/audit-engagements/{id}/export` - trigger export to IFMIS
- `GET /v1/governance/oag/audit-engagements/{id}/handover-receipt` - fetch OAG acknowledgment

Evidence packages are immutable once exported and signed by OAG; stored in long-term archive.

## Feature flags

`#[cfg(feature = "ifmis-integration")]` for live IFMIS API calls; tests use httpserver stub.

## Test commands

```bash
cargo test -p eel-governance-oag --all-features
cargo test -p eel-governance-oag --lib evidence_scope_validation
cargo test -p eel-governance-oag --test integration_oag -- --include-ignored
```

Mock IFMIS responses; verify evidence package structure and signatures.

## Bench commands

```bash
cargo bench -p eel-governance-oag export_package_1gb
```

Exporting a 1 GB evidence package should complete in < 30s.

## Dependencies of note

- `uuid6`: engagement and package IDs
- `tokio`: async IFMIS export, retry middleware
- `serde_json`: evidence scope and package serialization
- `ring`: HMAC evidence authentication tags

## Related docs

- `docs/api-reference.md#oag` (engagement lifecycle flowchart)
- IFMIS audit evidence exchange format spec (docs/external-specs/ifmis-aee.md)
- `migration_007_oag_engagements.sql` (schema: engagements, evidence_scopes, evidence_packages)
