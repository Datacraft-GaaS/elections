# Back-End Provisioning Expectation — Datacraft × IEBC Split

**Date:** 2026-08-21
**Status:** Draft, aligned with `DATACRAFT_SOFTWARE_SCOPE.md` v2 rewrite in progress
**Purpose:** capture the vendor-versus-client split for back-end provisioning under the Kenya IEBC IEMS 2026-2027 tender.

## Short Answer

Yes. Datacraft supplies, installs, provisions, and operates the entire back-end (hardware and software) for the 3-year framework, hosted physically at IEBC-designated facilities.

## Provisioning Split

| Provided by | Item |
|---|---|
| **Datacraft** (vendor) | All back-end hardware (ABIS cluster, HSMs, storage arrays, network gear, rack build-out) — see `HARDWARE-REQUIREMENTS-FROM-MITIGATIONS.md` Category H (10 items) and `PRICING-GUIDANCE.md` §3 |
| **Datacraft** | All software (28 modules M01-M28) as either Custom Software (vesting in IEBC on Operational Acceptance per Teaming Agreement §4.1), Standard Software (retained), OSS, or third-party pass-through |
| **Datacraft** | Installation, provisioning, testing, commissioning, cutover, 3-year steady-state operations, patching, backup, disaster recovery |
| **Datacraft** | Kenya-jurisdiction hosting design; on-prem racks at IEBC facilities; cloud fallback per reproducibility-bundle publication layer |
| **IEBC** (client) | Physical facilities: Anniversary Towers (primary DC), Bomas of Kenya (mirror), plus escrow site |
| **IEBC** | Power, physical security, HVAC, connectivity redundancy at facility |
| **IEBC** | Governance seats on Change Control Board, HSM key custody (three-key-custodian per Strategy §XVIII), signatures on Merkle-root publication |
| **IEBC** | Vote 2031 budget covers hosting-facility opex; Datacraft's KES 4.73bn FY2026/27 ICT sub-programme envelope covers vendor supply |

## What "Provisioned" Means in the Current Spec

Anchored in `DATACRAFT_SOFTWARE_SCOPE.md` Sections 2 (Reference architecture) and 4 (Cross-cutting foundations):

1. **Bare-metal Kubernetes cluster** on Kenyan-jurisdiction bare metal. Kubernetes 1.31, containerd runtime, Linkerd 2 service mesh, mTLS everywhere via SPIFFE / SPIRE identities.

2. **Three-site topology**:
   - Primary: Anniversary Towers
   - Mirror: Bomas of Kenya
   - Escrow: third IEBC site
   - Synchronous replication for primary ↔ mirror
   - Asynchronous replication for escrow

3. **HSM cluster** — Thales Luna 7 or Utimaco CryptoServer. FIPS 140-2 Level 3. Quorum-based key ceremony with three named custodians (IEBC Chair representative, Chief Justice representative, Auditor-General representative).

4. **Air-gapped provisioning environment** for reproducible builds. Nix or Bazel with locked-hash inputs. No internet access from the build environment.

5. **Immutable append-only audit-log storage** with 15-year retention per Elections Act evidentiary requirement.

6. **Vault + HSM-sealed secrets**. No plain-text secrets in git or CI/CD.

7. **Observability stack**: OpenTelemetry collector → Prometheus + Loki + Tempo + Grafana. Immutable retention 15 years for election-critical logs.

8. **Service mesh + identity**: Linkerd 2, mTLS by default, SPIFFE / SPIRE for workload identity, short-lived certs anchored to HSM-sealed root.

9. **Data protection**: Kenya-jurisdiction hosting, encryption-at-rest with TPM / HSM-sealed keys, encryption-in-transit TLS 1.3 with post-quantum hybrid (ML-KEM + X25519).

10. **API standards**: OpenAPI 3.1 spec-first, JSON Schema for events, semantic versioning.

## Contractual Anchor

The two-phase contracting architecture handles this as follows:

- **Phase 1 — Teaming Agreement**: commits both parties to the deliverables outlined here; IP boundary in §4 (Custom Software vests in IEBC on Operational Acceptance; Standard Software retained by Datacraft; OSS licensed direct).
- **Phase 2 — Subcontract Agreement**: contractualises every element of vendor obligation with SLAs, warranties, code escrow (monthly rolling deposits), and repository observer access.

## Cross-References

- `sw/DATACRAFT_SOFTWARE_SCOPE.md` — full subsystem specification (rewrite in progress at v2)
- `sw/PRD.md` — product-requirements decomposition (v2 rewrite in progress)
- `sw/ARCHITECTURE.md` — reference architecture
- `sw/SECURITY.md` — security posture
- `docs/research/HARDWARE-REQUIREMENTS-FROM-MITIGATIONS.md` — Category H back-end hardware requirements (10 items) and Category I Election Command Centre workstation requirements
- `docs/research/PRICING-GUIDANCE.md` §3 — back-end hardware pricing envelope
- `docs/research/CLIENT-EXCELLENCE-STRATEGY.md` Part XVIII — governance and compliance dashboards operated from within the back-end
- `mechrevo-datacraft-agreement/TEAMING-AGREEMENT.md` §4 — IP boundary and vesting

## Open Questions Still to Resolve Pre-Contract

- Confirm IEBC's designation of primary, mirror, and escrow sites (Anniversary Towers presumed but not formally confirmed for 2027 in public tender documents)
- Confirm IEBC's willingness to permit Kenya-jurisdiction cloud fallback (e.g., Konza / iColo / Safaricom Kenya DC) for the reproducibility-bundle publication layer only, versus 100% on-prem
- Confirm the three named HSM key custodians (Chair, CJ representative, AG representative) and the key-ceremony logistics
- Confirm sub-second replication SLA between primary and mirror is achievable across the IEBC facility uplinks
- Confirm 15-year retention storage cost is Datacraft-borne or IEBC-borne after Operational Acceptance
