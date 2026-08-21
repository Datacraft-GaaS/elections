# API Contracts

All OpenAPI 3.1 contracts live under `packages/openapi-contracts/`. One directory per API domain; each contains `openapi.yaml`.

## Contracts

| Contract | File path | Owning service | Description |
|---|---|---|---|
| abis | `packages/openapi-contracts/abis/openapi.yaml` | abis-svc | Biometric match requests, adjudication queue, enclave attestation |
| ai-anomaly | `packages/openapi-contracts/ai-anomaly/openapi.yaml` | ai-platform-svc | Anomaly detection alerts query |
| ai-forecast | `packages/openapi-contracts/ai-forecast/openapi.yaml` | ai-platform-svc | Turnout forecast publications |
| ai-redteam | `packages/openapi-contracts/ai-redteam/openapi.yaml` | ai-platform-svc | Red-team engagement management |
| ai-registry | `packages/openapi-contracts/ai-registry/openapi.yaml` | ai-platform-svc | AI model registry and model cards |
| ballot-serial | `packages/openapi-contracts/ballot-serial/openapi.yaml` | rts-svc | Ballot serial chain: assignment, receipts, reconciliation |
| biometric-edge | `packages/openapi-contracts/biometric-edge/openapi.yaml` | biometric-edge-svc | Kit sync, enrolment upload, session management |
| boundary | `packages/openapi-contracts/boundary/openapi.yaml` | boundary-svc | Boundary query, fairness metrics, geofence lookup |
| bvi | `packages/openapi-contracts/bvi/openapi.yaml` | bvi-svc | Kit config distribution, session management, turnout |
| bvr | `packages/openapi-contracts/bvr/openapi.yaml` | bvr-svc | Voter lookup, template management, register snapshots |
| crms | `packages/openapi-contracts/crms/openapi.yaml` | crms-svc | Candidate nomination and clearance workflow |
| credential | `packages/openapi-contracts/credential/openapi.yaml` | bvr-svc | Voter credential issuance |
| custody | `packages/openapi-contracts/custody/openapi.yaml` | inventory-svc | Physical custody chain query |
| cvr | `packages/openapi-contracts/cvr/openapi.yaml` | bvr-svc | Continuous voter register update ingest |
| declaration | `packages/openapi-contracts/declaration/openapi.yaml` | rts-svc | Result declaration and Certificate of Election |
| dereg | `packages/openapi-contracts/dereg/openapi.yaml` | bvr-svc | Voter deregistration and appeal |
| diaspora-logistics | `packages/openapi-contracts/diaspora-logistics/openapi.yaml` | inventory-svc | Diaspora ballot logistics and customs |
| ecc | `packages/openapi-contracts/ecc/openapi.yaml` | ecc-svc | ECC dashboard, alerts, S.44A |
| explainer | `packages/openapi-contracts/explainer/openapi.yaml` | portal-svc | Referendum explainer content |
| field-mobile-bff | `packages/openapi-contracts/field-mobile-bff/openapi.yaml` | field-mobile-svc | Citizen verification app BFF |
| field-mobile-dispatch | `packages/openapi-contracts/field-mobile-dispatch/openapi.yaml` | field-mobile-svc | Personnel dispatch management |
| field-mobile-support | `packages/openapi-contracts/field-mobile-support/openapi.yaml` | field-mobile-svc | Field support tickets |
| gazette | `packages/openapi-contracts/gazette/openapi.yaml` | portal-svc | Gazette notices and election calendar |
| governance | `packages/openapi-contracts/governance/openapi.yaml` | governance-svc | Governance overview and referrals |
| governance-by-election | `packages/openapi-contracts/governance-by-election/openapi.yaml` | governance-svc | By-election operations |
| governance-county-coord | `packages/openapi-contracts/governance-county-coord/openapi.yaml` | governance-svc | County/national coordination |
| governance-court | `packages/openapi-contracts/governance-court/openapi.yaml` | governance-svc | Election court evidence bundles |
| governance-forensics | `packages/openapi-contracts/governance-forensics/openapi.yaml` | governance-svc | Financial forensics analysis |
| governance-intl-observer | `packages/openapi-contracts/governance-intl-observer/openapi.yaml` | governance-svc | International observer coordination |
| governance-legal-audit | `packages/openapi-contracts/governance-legal-audit/openapi.yaml` | governance-svc | Transmission legal audit |
| governance-recall | `packages/openapi-contracts/governance-recall/openapi.yaml` | governance-svc | Recall petition management |
| governance-referendum | `packages/openapi-contracts/governance-referendum/openapi.yaml` | governance-svc | Referendum mechanics |
| governance-reporting | `packages/openapi-contracts/governance-reporting/openapi.yaml` | governance-svc | Parliamentary and judiciary reports |
| governance-review | `packages/openapi-contracts/governance-review/openapi.yaml` | governance-svc | Post-election review findings |
| governance-rla | `packages/openapi-contracts/governance-rla/openapi.yaml` | governance-svc | Risk-limiting audit runs |
| grievance | `packages/openapi-contracts/grievance/openapi.yaml` | ecc-svc | Grievance intake and tracking |
| iab | `packages/openapi-contracts/iab/openapi.yaml` | ecc-svc | IAB escalations and reports |
| inspection | `packages/openapi-contracts/inspection/openapi.yaml` | portal-svc | Voter list inspection |
| inventory | `packages/openapi-contracts/inventory/openapi.yaml` | inventory-svc | Asset management, custody, MDM |
| ivr | `packages/openapi-contracts/ivr/openapi.yaml` | field-mobile-svc | IVR channel gateway |
| nomination | `packages/openapi-contracts/nomination/openapi.yaml` | crms-svc | Nomination fee payment |
| party | `packages/openapi-contracts/party/openapi.yaml` | crms-svc | Party registration, primaries, finance submission |
| portal | `packages/openapi-contracts/portal/openapi.yaml` | portal-svc | Public result query and inclusion proofs |
| printing | `packages/openapi-contracts/printing/openapi.yaml` | inventory-svc | Ballot print orders |
| queue | `packages/openapi-contracts/queue/openapi.yaml` | queue-svc | Queue snapshots and S.14A |
| rts | `packages/openapi-contracts/rts/openapi.yaml` | rts-svc | Form submission, ICR status, publication |
| support-archive | `packages/openapi-contracts/support-archive/openapi.yaml` | support-admin-svc | Long-term BagIt archive |
| support-budget | `packages/openapi-contracts/support-budget/openapi.yaml` | support-admin-svc | Budget and procurement |
| support-intent | `packages/openapi-contracts/support-intent/openapi.yaml` | support-admin-svc | Intent preservation and adjudication |
| support-payroll | `packages/openapi-contracts/support-payroll/openapi.yaml` | support-admin-svc | HR and payroll |
| support-special-voter | `packages/openapi-contracts/support-special-voter/openapi.yaml` | support-admin-svc | Special voter facilities |
| symbol | `packages/openapi-contracts/symbol/openapi.yaml` | crms-svc | Party symbol and colour registry |
| training-accreditation | `packages/openapi-contracts/training-accreditation/openapi.yaml` | training-svc | Media and observer accreditation |
| training-cms | `packages/openapi-contracts/training-cms/openapi.yaml` | training-svc | Voter education content management |
| training-lms | `packages/openapi-contracts/training-lms/openapi.yaml` | training-svc | Poll-worker LMS and certification |
| training-medical | `packages/openapi-contracts/training-medical/openapi.yaml` | training-svc | Medical first-response dispatch |
| transparency | `packages/openapi-contracts/transparency/openapi.yaml` | transparency-svc | Bundle query and token verification |
| ussd | `packages/openapi-contracts/ussd/openapi.yaml` | field-mobile-svc | USSD channel gateway |
| verifier | `packages/openapi-contracts/verifier/openapi.yaml` | bvi-svc | Public voter verification lookup |
| voter-card | `packages/openapi-contracts/voter-card/openapi.yaml` | bvr-svc | Voter ID card batch management |
| vprs | `packages/openapi-contracts/vprs/openapi.yaml` | vprs-svc | Voter pre-registration |
| whatsapp | `packages/openapi-contracts/whatsapp/openapi.yaml` | field-mobile-svc | WhatsApp webhook |

## Shared Schema

`packages/openapi-contracts/_shared/` contains common components (error envelopes, pagination, PII-tier headers) referenced by all contracts.

## Client Generation

See [clients.md](clients.md) for how to regenerate language clients from these contracts.
