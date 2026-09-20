# G14 — Election Command Centre (ECC) Playbook (CLOSED)
## Physical Setup
- Location: IEBC HQ, Nairobi + 2 hot-sites (Kisumu, Mombasa)
- Floor: 3 zones — Operations, Comms, Legal/IAB
- Displays: 12 screens (4 per zone) showing real-time dashboards

## Network
- Primary: Kenya Education Network (KENET) — 1 Gbps dedicated
- Backup 1: Safaricom 5G failover (auto on primary loss)
- Backup 2: Starlink BGAN (manual activation)
- Data-diode: Cilium egress-only enforced on kit namespaces

## Comms Redundancy
- Primary: IEBC SMS gateway (integrations/telco-sms)
- Backup: WhatsApp Business API (integrations/whatsapp)
- Tertiary: VHF radio net (manual tally protocol)

## Key Dashboards (Grafana, platform/observability/dashboards/)
1. kit-heartbeat.json — % kits online, battery, signal
2. transmission-status.json — Form 34A submitted / pending / failed
3. anomaly-alert.json — UEBAnomalyDetector output (platform/detection/ueba/)
4. incident-feed.json — all open incidents by severity

## Escalation
- P1: 15 min → IAB + PM + Legal
- P2: 30 min → Security Lead + CCB
- P3: 2h → Operations Lead
- Section 44A: Immediate → CCB convenes

## M5 Audit
Every ECC action emits audit event (auditId=g14-ecc-{action}, proofHash, signature).
