# ADR 0008: Portal stack

Status: accepted
Date: 2026-08-21

## Context

Portals in this system are three distinct shapes: static-first public read surfaces, internal admin CRUD, and real-time public-exposed dashboards. Under adversarial pressure from millions of concurrent viewers, one-size-fits-all is a security and performance mistake. Next.js was the original candidate and is rejected due to a recent CVE record — CVE-2025-29927 (middleware auth bypass), CVE-2024-46982 (cache poisoning), CVE-2024-34351 (SSRF via server actions) — plus Vercel-driven direction and a weak on-prem story. See `docs/architecture/decision-log.md` §3 Phase 6-8.

## Decision

Three portal frameworks + Tauri + Flutter, allocated strictly by portal category (Path B).

| shape | modules | framework |
|---|---|---|
| Static-first public read | M04 VPRS, M07 BVV, M11 static shell, M18 boundary viewer, M22 finance viewer, M37 voter-ed CMS, M52 gazette, M64 voter-list inspection | **Astro 5** + Svelte islands where interactive |
| Internal admin / backoffice / form CRUD | M12 CRMS + party, M42/M43 accreditation admin, M45 party mgmt, M35/M36 register admin | **Django 5 + HTMX + Alpine** |
| Real-time public-exposed dashboards | M11 dynamic parts, M13 real-time, M15 IAB dashboard, M19 queue transparency, M23 grievance channels, M26 anomaly triage | **Phoenix LiveView** (Elixir) |
| Operator desktop | M09 RO tally, M16 Command Centre (LiveView inside a Tauri webview), M21 audit laptops | **Tauri 2** (Rust + web UI) |
| Kits + mobile apps | M01, M05, M08, M13, M24, M25, M27, M29-M34 | **Flutter/Dart** |

Rules:

- All portals verify JWT against Keycloak JWKS via the `iebc-auth` shared library. Same auth pattern in Rust services, Django views, LiveView hooks, Astro server routes.
- BFF discipline (ADR-0007): browsers never see refresh tokens or call SpiceDB/OpenBao directly.
- Strict CSP with per-request nonces; no external JS ever; no CDN fetches — everything self-hosted via Harbor/nginx/MinIO.
- Astro static builds published as cosign-signed tarballs to MinIO; served through the edge (ADR-0010).
- LiveView runs behind Cilium mesh mTLS with SPIFFE identity; scaled horizontally with distributed Erlang cluster (libcluster + K8s selector); Phoenix.PubSub as fan-out spine.
- Tauri desktop uses Rust core for HSM/PKCS#11 access on RO tally + audit laptops; web UI in the webview.

## Consequences

Positive: each framework matches its portal category natively; LiveView removes browser-JWT attack surface for exposed real-time surfaces; Django is mature and audit-defensible for admin; Astro ships mostly HTML for public read at maximum cache-friendliness; smallest possible client bundle per surface.

Negative: three portal frameworks is real cost — three build tool chains, three CI lanes, three teams' worth of expertise. Mitigated by strict per-category allocation, shared design tokens across the three, and identical auth/audit libraries. Elixir hiring pool in Kenya is thinner (see ADR-0002 mitigation).

## Alternatives rejected

- **Next.js** — CVE record + Vercel direction + poor on-prem story.
- **Remix / React Router 7** — better auth patterns than Next but no decisive win over LiveView + SvelteKit; React ecosystem risks persist.
- **SvelteKit** — solid; superseded by LiveView for real-time surfaces once the fourth-language cost was accepted.
- **SolidStart** — technically excellent; framework maturity for a 3-year contract insufficient.
- **Path A** (Astro + Django + SvelteKit) — rejected once user constraint escalated to "millions of viewers under adversarial pressure"; LiveView is the technically superior real-time substrate at that scale.
- **Path C** (LiveView-maximal, drop Django) — LiveView on admin CRUD does not earn its cost against Django's maturity.

## Related

ADR-0002 (languages), ADR-0007 (BFF pattern), ADR-0009 (portals inherit degradation modes), ADR-0010 (edge fronts all portals), ADR-0011 (portals consume the public API tier).
