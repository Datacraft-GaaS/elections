# eel-web

Django reusable app for the eel platform.

## Features

- `EelAuthMiddleware`: validates Keycloak JWT Bearer tokens, populates `request.eel_subject`
- `EelAuditMiddleware`: logs requests with subject info
- `EelPiiScrubberMiddleware`: strips PII from exception reports and log messages
- `SpiceDBBackend`: Django auth backend delegating permissions to SpiceDB
- Transactional outbox model (`OutboxRecord`)
- Idempotency key store (`IdempotencyKey`)
- Event deduplication (`DeduplicatedEvent`)
- `AuditShadowMixin`: audit history via signals
- PII-aware form fields (`PIIRestrictedField`, `PIISensitiveField`, `PIIOperationalField`)
- HTMX-ready base template
- `eel_seed_realms` management command

## Install

```
pip install eel-web
```

Add to `INSTALLED_APPS`:

```python
INSTALLED_APPS = [
    ...
    "eel_web",
]
```
