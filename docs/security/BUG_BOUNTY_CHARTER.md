# G17 — Bug Bounty Programme Charter (CLOSED)
## Scope
- In-scope: all 20 EEL services + integrations + platform layer
- Out-of-scope: physical hardware tampering, social engineering, DDoS

## Reward Tiers (USD)
- Critical (RCE, vote manipulation, data exfil): $10,000–$25,000
- High (privilege escalation, audit tampering): $5,000–$10,000
- Medium (XSS, CSRF on admin): $1,000–$3,000
- Low (info disclosure, minor DoS): $100–$500

## Rules of Engagement
- No testing on production without written authorisation
- 90-day responsible disclosure window
- Safe-harbour: good-faith researchers exempt from prosecution
- Public disclosure only after patch deployed + 14-day grace

## M5 Integration
Every submission emits auditId=g17-bug-{cve-or-internal-id}, reviewed by security team.
