# ADR 0020: Service topology — 18 K8s services grouping 69 modules

Status: accepted
Date: 2026-08-21

## Context

Sixty-nine module specs do not equal sixty-nine services. `sw/ARCHITECTURE.md` §2 (v3) groups the 69 modules into 18 Kubernetes services along bounded-context and shared-lifecycle lines. This grouping is Datacraft-drafted (not RFP-binding) but it aligns cleanly with the T1/T2 data topology in ADR-0012 and is the shape ArgoCD ApplicationSets will fan out over.

## Decision

**Eighteen K8s services, mapping N modules each.**

| service | modules | notes |
|---|---|---|
| `biometric-edge-svc` | M01 BVR-device, M05 BVI-device (edge-side APIs supporting kit clients) | edge-side of biometric enrolment / identification |
| `bvr-svc` | M02 BVR-backend, M35 CVR, M36 deregistration-appeal, M53 voter-ID-card | voter register lifecycle |
| `abis-svc` | M03 ABIS | confidential-compute enclave (see ADR-0005 amendment) |
| `vprs-svc` | M04 VPRS | public self-preregistration |
| `bvi-svc` | M06 BVI-backend, M07 BVV | polling-day identification + register inspection |
| `rts-svc` | M08 RTS-device edge API, M09 RTS-RO, M10 RTS-backend, M51 declaration, M63 ballot-serial-chain | results transmission end-to-end; T1-consensus cluster |
| `portal-svc` | M11 public portal, M13 verifier, M52 gazette, M64 voter-list-inspection, M68 referendum-explainer | public web surfaces (LiveView + Astro; see ADR-0008) |
| `crms-svc` | M12 CRMS + party portal, M45 party-reg, M46 primaries, M47 nomination-fees, M67 symbol/colour-registry | candidate + party lifecycle |
| `transparency-svc` | M14 reproducibility-bundle, M17 speech-integrity, M18 boundary-transparency, M22 finance-transparency, transparency log (ADR-0017) | verifiability + integrity surfaces |
| `ecc-svc` | M15 IAB dashboard, M16 command-centre, M19 queue-transparency, M23 grievance-channels | operator + independent oversight |
| `boundary-svc` | M18 boundary maps | shared geo store, PostGIS + OpenSearch |
| `queue-svc` | M19 queue transparency | real-time queue telemetry pipeline |
| `inventory-svc` | M20 chain-of-custody, M39 ballot-printing-chain, M65 diaspora-logistics | RFID/GPS + Merkle-chained custody |
| `governance-svc` | M21 RLA-audit, M40 court-interface, M41 post-election-review, M48 recall, M49 referendum-mechanics, M50 by-election, M56 county-coord, M57 international-observer-coord, M58 parliamentary-reporting, M59 OAG-interface, M62 transmission-legal-audit, M66 forensics | oversight + legal + audit surfaces |
| `ai-platform-svc` | M17 speech, M26 anomaly, M28 AI-monitoring, M44 turnout-forecasting, M61 secrecy-assurance | ML inference (ONNX Runtime, Python tier per ADR-0002); Rust gateway in front |
| `field-mobile-svc` | M13, M29, M30, M31, M32, M33, M34 backends | mobile-suite BFF + shared verifier gateway |
| `training-svc` | M37 voter-ed CMS, M38 LMS, M42 media-accreditation, M43 observer-accreditation, M60 medical-response | Django admin surfaces (ADR-0008); shared T2 cluster |
| `support-admin-svc` | M20 admin ops, M23 grievance backoffice, M25 special-voter workflows, M27 intent-preservation, M54 budget/expenditure, M55 HR/payroll, M69 long-term archive | internal admin + finance + archive (BagIt) |

Total: 18 services, 69 modules. **Cardinality is a design choice, not a physical constraint** — a service may be split later as an ADR change if scale demands.

## How this shapes the scaffold

- **`services/<service-name>/` at repo root** (18 directories), each a self-contained deployable unit.
- **`modules/<module-slug>/` remains** as the design/spec/doc home for each of the 69 modules. Cross-references from `services/<name>/README.md` list the modules it implements.
- **`platform/argocd/applicationsets/` fans over `services/`**, one ApplicationSet per service, one Application per (service × site).

## Consequences

Positive: 18 units of deployment is manageable; each service owns a coherent bounded context; ArgoCD fans out cleanly; ownership is clear (one team per service or per small group of services).

Negative: some services (governance-svc, transparency-svc) span many modules — internal boundary discipline required. A service can be split as needed via new ADR without cross-cutting cost.

## Alternatives rejected

- One service per module (69 services): explodes ops surface for no design win.
- One service total: destroys blast-radius isolation and shared-team ownership; incompatible with the T1/T2 topology in ADR-0012.
- Grouping by data class (all T1 in one service, all T2 in another): breaks bounded-context ownership.

## Related

ADR-0001 (module directories persist as design homes), ADR-0004 (K8s hosts these), ADR-0012 (data topology aligns to T1/T2 boundaries visible here), ADR-0013 (event-sourced modules cluster in rts-svc, inventory-svc, transparency-svc).
