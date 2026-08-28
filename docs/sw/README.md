# IEBC IEMS — Software Product Documentation

**Tender:** IEBC/OIT/01/2026-2027 — Integrated Elections Management System
**Bidder:** Mechrevo (principal, hardware SI) + Datacraft (named subcontractor, software prime)
**Software owner:** Datacraft Limited
**IPR:** Vests in IEBC per GCC 15.4 on Operational Acceptance for all Custom Software; perpetual irrevocable licence to IEBC for Standard Software per GCC 16.

## Document Map

| File | Purpose |
|---|---|
| `PRD.md` | Master Product Requirements Document. Vision, scope, personas, cross-cutting NFRs, per-module functional requirements, acceptance criteria. Primary reference. |
| `ARCHITECTURE.md` | System architecture: deployment topology, data flow, service boundaries, technology choices, resilience patterns. |
| `SECURITY.md` | Consolidated security requirements: threat model reference, controls, compliance to ISO 27001:2022 + ISO 17799 + Kenya DPA 2019 + Elections (Technology) Regulations 2017. |
| `DATA-MODEL.md` | Canonical data model: entities, relationships, cardinalities, retention, immutability guarantees. |
| `API-CATALOG.md` | Complete API inventory: endpoints, contracts, authentication, versioning, rate limits. |
| `ACCEPTANCE-CRITERIA.md` | Consolidated Factory / Site / User / Regulatory Acceptance Test criteria. |
| `modules/` | One PRD per core module (BVR, ABIS, VPRS, BVI, BVV, RTS, Public Portal, CRMS, Inventory, MDM, IAM/PKI, Audit). |

## Reading Order

1. Start with **PRD.md** for the full picture.
2. **ARCHITECTURE.md** for how it fits together.
3. **SECURITY.md** for the defensive posture.
4. **modules/*.md** for implementation-level detail per module.
5. **DATA-MODEL.md** + **API-CATALOG.md** as living references during build.
6. **ACCEPTANCE-CRITERIA.md** to close the loop at delivery.

## Cross-References

- `../THREAT_MODEL.md` — the sovereign-grade threat model this software must implement.
- `../DATACRAFT_SOFTWARE_SCOPE.md` — enumerated 42-artefact build manifest.
- `../TENDER_ANALYSIS.md` — tender clause map.
