# Threat Model - {{cookiecutter.svc_title}}

Data class: {{cookiecutter.data_class}}

## Assets

- (list key data assets)

## Threats

| Threat | Mitigation |
|---|---|
| Unauthorized access | EelAuthMiddleware JWT validation |
| Replay attacks | IdempotencyKey table |
| Data leakage | PII scrubbing middleware |
