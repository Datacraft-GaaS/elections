# SA-002: Portal stack: Path B (Astro + Django + LiveView + Tauri + Flutter) vs Next.js 14

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/the Commission/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §18 pins **Next.js 14** as the SSR/SSG framework for public portals (M11 public portal, M13 verifier, M29 citizen verify, M68 referendum explainer).

## Decision (per ADRs)

Replace with a per-portal-shape allocation (Path B):

| shape | modules | framework |
|---|---|---|
| Static-first public read | M04, M07, M11 static shell, M18 viewer, M22 viewer, M37, M52, M64 | **Astro 5** + Svelte islands |
| Internal admin CRUD | M12, M42/M43, M45, M35/M36 | **Django 5 + HTMX + Alpine** |
| Real-time public dashboards | M11 dynamic, M13 real-time, M15, M19, M23, M26 | **Phoenix LiveView** |
| Operator desktop | M09, M16 (LiveView in Tauri), M21 | **Tauri 2** |
| Mobile + kits | M01, M05, M08, M13, M24, M25, M27, M29-M34 | **Flutter/Dart** |

## RFP status

**Not RFP-binding.** `tender.txt` mandates "API-based data exchange" (L2217) and RBAC + MFA for portal access; no framework named.

## Rationale

1. **Next.js CVE record over the past 18 months is disqualifying for an election-scale system:**
   - **CVE-2025-29927** (March 2025), critical middleware authentication bypass via `x-middleware-subrequest` header. Any auth check in Next.js middleware bypassable by adding one header.
   - **CVE-2024-46982**, cache poisoning via SSR.
   - **CVE-2024-34351**, SSRF via server actions.
   - The middleware-as-auth-boundary pattern has been fundamentally questioned by the security community.
2. **Vercel-driven direction**. Next.js's roadmap prioritises Vercel-hosted features (edge runtime, image optimisation, ISR). On-prem deployments are second-class.
3. **The portals are not one shape.** Static content, admin CRUD, and real-time dashboards have different threat models and different optimal frameworks. One-size-fits-all imports every framework's worst-case surface into every portal.
4. **LiveView is the right substrate for the storm-facing surfaces.** Server-authoritative state removes browser JWT + localStorage exfiltration vectors entirely. BEAM per-connection process isolation limits blast radius to one session on hostile input. Discord runs ~5M concurrent per node on this stack. Election night is exactly the workload.
5. **Django is the right substrate for admin.** 20 years of security-hardened CRUD, auditor-defensible, small attack surface for form-driven internal work.
6. **Astro serves signed-snapshot static content at near-zero client JS.** Perfect fit for M11 public portal's CDN-cached signed-tarball model (ADR-0011 Tier 1).

## Trade-offs

- Three portal frameworks is more surface than one. Real cost: three build toolchains, three CI lanes, three teams' worth of expertise. Mitigated by strict per-category allocation, shared design tokens across all three, and identical `eel-auth` shared library making the auth code identical.
- Elixir is a fourth backend language. Real hiring cost in Kenya; mitigated by 2-3 senior anchors + upskilling.
- LiveView-in-Tauri-webview for M16 Command Centre is unusual; validated in prior deployments (e.g. supabase-realtime tooling).

## Edits to apply to sw/

None. Next.js is not RFP-binding (tender.txt names no portal framework) and is excluded from this programme. Path B is the build target; no Next.js component ships in any tier.

## Related

ADR-0008 (portal stack), ADR-0007 (auth pattern), ADR-0011 (Tier 1 signed snapshots), ADR-0023 (RFP-alignment).
