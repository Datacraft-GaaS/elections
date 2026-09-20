# G22 — SMS Short-Code Provisioning (CLOSED)
## Provider: Safaricom (primary), Airtel (backup)
## Short Code: 20822 (dedicated to IEBC election results queries)
## Service: USSD (*384#) + SMS query (short code 20822)
## Agreement Terms
- Contract term: 24 months (renewable)
- Service Level: 99.9% uptime, <2s response time
- Monthly fee: KES 85,000 (primary); KES 65,000 (backup)
- Integration: REST + SMPP
- M5 audit: every SMS event (query/response) emits auditId=g22-sms, proofHash, sig
