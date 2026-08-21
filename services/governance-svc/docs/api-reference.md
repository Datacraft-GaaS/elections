# governance-svc API reference (WP-5.1)

OpenAPI contracts in packages/openapi-contracts/governance-{rla,court,legal-audit,reporting}/. All endpoints: JWT + mTLS, tier 3. Write operations: Idempotency-Key header required.

## RLA (port 8091)
- GET /api/v1/governance/rla/audits
- POST /api/v1/governance/rla/audits
- GET /api/v1/governance/rla/audits/{run_id}
- GET /api/v1/governance/rla/audits/{run_id}/samples
- POST /api/v1/governance/rla/audits/{run_id}/counts
- GET /api/v1/governance/rla/audits/{run_id}/report

## Court Interface (port 8092)
- GET/POST /api/v1/governance/court/petitions
- GET /api/v1/governance/court/petitions/{petition_id}
- GET/POST /api/v1/governance/court/petitions/{petition_id}/evidence-bundles
- GET /api/v1/governance/court/petitions/{petition_id}/witnesses
- POST /api/v1/governance/court/petitions/{petition_id}/witnesses/pre-register

## Legal Audit (port 8093)
- GET/POST /api/v1/governance/legal-audit/transmissions
- GET /api/v1/governance/legal-audit/verdicts
- GET /api/v1/governance/legal-audit/party-summary

## Reporting (port 8094)
- GET/POST /api/v1/governance/reporting/parliamentary-reports
- GET /api/v1/governance/reporting/parliamentary-reports/{report_id}
- GET/POST /api/v1/governance/reporting/oag-submissions

## OAG (port 8095)
- GET/POST /api/v1/governance/oag/engagements
- GET /api/v1/governance/oag/engagements/{engagement_id}
- POST /api/v1/governance/oag/engagements/{engagement_id}/extracts
- GET/POST /api/v1/governance/oag/engagements/{engagement_id}/responses
- GET/POST /api/v1/governance/oag/published-reports
