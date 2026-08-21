# Wave 6: Full-system verification

Part of the IEBC IEMS full build plan. See `README.md` for conventions, `08-edge-case-matrix.md` for EC-IDs, `10-definition-of-done.md` for the WP exit bar.


Gate G6 (IEBC OIT + architect + security + IAB + legal): all items green; sign-off recorded as tlog meta-event.

| WP | Name | Done-criteria | Size |
|---|---|---|---|
| 6.1 | Election-day rehearsal | Pre-prod full topology (2 sites + 5 edges); 65k kits, 60k RTS, 18k req/s BVI, 5M viewers/hr, 250k check-ins, 10M verifier; every Appendix-A numeric SLA met; witness cross-check; declaration + portal + verifier + bundle consistent | L |
| 6.2 | DR failover drill | rts-svc RTO ≤30 min / RPO ≤5 min; other T1 ≤60/15; T2 ≤4 h/1 h; edge region ≤15 min; restore from pgBackRest/Velero/Kopia; restore itself logged to tlog | M |
| 6.3 | Chaos suite | All 5.9 scenarios during 6.1 load; no SLO breach beyond degradation contract | M |
| 6.4 | 5x load | 5x peak write path; P99 25 ms end-to-end (ADR-0018); Kafka 100k ev/s + headroom; regression gate report | M |
| 6.5 | Security harness | SAST/SCA/container clean; ZAP + Nuclei DAST via edge; 24 h fuzz on all §7 boundaries; pen-test harness + findings tracker; ASVS L3 evidence complete; testssl.sh PQ-hybrid | L |
| 6.6 | Accessibility + i18n sweep | WCAG 2.2 AA on every UI, AAA on M68; 11 locales + pseudo-locale; NVDA/TalkBack/VoiceOver scripts on kits, mobile, portals, desktop | M |
| 6.7 | Completeness audit | §9 procedure; zero unmapped rows; report + signed attestation to tlog | M |
