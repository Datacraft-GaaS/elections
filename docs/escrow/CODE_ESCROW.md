# G15 — Code Escrow Agreement (CLOSED)
## NCC Group Dual-Vault Escrow
- Vault A: Nairobi datacentre (Equinix NBO-1)
- Vault B: London datacentre (Interxion LON-1)
- Deposit schedule: weekly (every Friday 18:00 UTC) during T-180 to T+30
- Release triggers: IEBC written notice OR 30-day escrow agent inactivity OR court order
- Verification: reproduce-build.sh script in vault; must match signed SBOM

## Contents
- Full source code (all 20 services)
- Build artifacts (Docker images, APK, .deb)
- SBOM (SPDX 2.3, generated at build time)
- Cryptographic keys (HSM config, not raw keys)
- Configuration templates (non-secret)
- M5 audit log excerpt (last 90 days)

## M5 Audit
Source deposit emits auditId=g15-escrow-deposit-{date}, proofHash, signature.
