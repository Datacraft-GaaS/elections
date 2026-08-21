# services/

Eighteen Kubernetes services grouping the 69 modules per ADR-0020. Each subdirectory is a self-contained deployable unit.

| service | primary language | modules |
|---|---|---|
| biometric-edge-svc | Rust + Flutter (kits) | M01, M05, M24 |
| bvr-svc | Rust | M02, M35, M36, M53 |
| abis-svc | Rust + confidential compute (SEV-SNP/TDX) | M03 |
| vprs-svc | Rust | M04 |
| bvi-svc | Rust | M06, M07 |
| rts-svc | Rust (event-sourced core) | M08, M09, M10, M51, M63 |
| portal-svc | Astro + LiveView | M11, M52, M64, M68 |
| crms-svc | Django (admin) + Rust (API) | M12, M45, M46, M47, M67 |
| transparency-svc | Rust (HSM-integrated) | M14, M17, M18, M22, transparency log |
| ecc-svc | LiveView | M15, M16, M19, M23 |
| boundary-svc | Rust + PostGIS | M18 shared geo store |
| queue-svc | Rust + LiveView fan-out | M19 |
| inventory-svc | Rust (event-sourced) | M20, M39, M65 |
| governance-svc | Rust + Python | M21, M40, M41, M48, M49, M50, M56-M59, M62, M66 |
| ai-platform-svc | Python (ONNX Runtime) | M17, M26, M28, M44, M61 |
| field-mobile-svc | Rust (BFF) + Flutter (M13, M29-M34) | M13, M29-M34 |
| training-svc | Django | M37, M38, M42, M43, M60 |
| support-admin-svc | Django + Rust | M20 ops, M25, M27, M54, M55, M69 |

Each service has:
- `src/` — implementation
- `tests/` — unit + integration
- `migrations/` — schema migrations (Refinery / sqlx / Django / Ecto per language)
- `deploy/` — Kubernetes manifests, ArgoCD Application, Helm chart if used
- `docs/` — service-specific docs (README, threat model, event catalog)
