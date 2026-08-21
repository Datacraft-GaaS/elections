# Threat model: {{project-name}}

Tier: {{tier}}

## Assets

- (list key data assets here)

## Threats

| ID | Threat | Mitigation |
|---|---|---|
| T-01 | Unauthorized access | JWT + SpiceDB RBAC |
| T-02 | Data tampering | Cosign signatures on all responses |
| T-03 | Replay attacks | Idempotency-Key deduplication |
