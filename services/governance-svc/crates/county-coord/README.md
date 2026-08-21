# eel-governance-county-coord

M56 County-national coordination: federated SAML authentication, MoU tracking, ministry approvals, incident escalation.

## What it does

Manages election-time coordination between national IEBC and 47 county governments, plus integration with key ministries (Interior, Infrastructure, Police). Tracks Memoranda of Understanding, approval workflows for resource allocation and security, and incident escalation chains. Uses SAML federation to authenticate county staff via each county's IdP, mapping attributes to roles (county_returning_officer, county_admin, county_observer).

Fits in governance-svc as the county liaison layer. Runs on port 8093 (shared service); integrates with SpiceDB for fine-grained county-level authz (county_id in resource namespace).

## Public API

`eel_governance_county_coord::coordination::CountyCoordination` aggregate: `fn establish_mou(&mut self, county_id: str, mou: Mou) -> Result<()>`, `fn request_ministry_approval(&mut self, approval_type: ApprovalType) -> Result<()>`, `fn escalate_incident(&mut self, incident: CoordinationIncident) -> Result<>`.

Endpoints:
- `GET /v1/governance/county-coord/mou/{county_id}` - fetch MoU status
- `POST /v1/governance/county-coord/mou/{county_id}` - establish MoU
- `POST /v1/governance/county-coord/approvals/{approval_id}` - respond to approval request
- `GET /v1/governance/county-coord/incidents/{county_id}` - list escalated incidents

All endpoints require valid SAML assertion + SpiceDB check (user.county_id == resource.county_id or user.role == "national_admin").

## Feature flags

`#[cfg(feature = "saml-federation")]` for live SAML IdP metadata fetch; tests use xmlsec3 stub.

## Test commands

```bash
cargo test -p eel-governance-county-coord --all-features
cargo test -p eel-governance-county-coord --lib saml_attribute_mapping
cargo test -p eel-governance-county-coord --test integration_county_coord -- --include-ignored
```

Mock SAML assertions via test fixtures; verify authz isolation per county.

## Bench commands

```bash
cargo bench -p eel-governance-county-coord mou_establish
```

MoU establishment and signature verification should complete in < 200ms.

## Dependencies of note

- `xmlsec3`: SAML signature verification
- `uuid6`: MoU and incident IDs
- `tokio`: async SAML metadata fetch and refresh
- `chrono`: MoU expiry tracking

## Related docs

- `docs/api-reference.md#county-coord` (SAML flowchart, attribute mappings)
- `docs/architecture.md#county-federation` (federated IdP setup diagram)
- `migration_005_county_coordination.sql` (schema: mous, approvals, incidents)
