# G10 — 24x7 SOC Runbook (CLOSED)
# Alert tiers: P1 (critical) → P2 (high) → P3 (medium) → P4 (low)
## P1 (Kit/RTS/DB/HSM failure): Escalate to IAB + PM within 15 min; activate manual protocol
## P2 (Intrusion / gas / DDoS): Escalate to security lead; CCB convened within 30 min
## P3 (Form quality / voter lookup fail): Monitor; fix within 2h; no escalation
## P4 (Minor telemetry gap): Log; fix within 24h
# On-call roster: 4 SOC analysts + 1 security lead + 1 IAB observer (read-only access)
# M5 audit: every alert → correlation_key → detection rule → playbook reference
