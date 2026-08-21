# eel-auth-py

JWT/JWKS verification, SpiceDB authorization, and audit logging for the eel platform.

## Features

- JWKS caching with TTL and kid-miss refresh
- JWT verification via python-jose
- Subject extraction from Keycloak JWT claims
- SpiceDB permission checks and resource lookups (HTTP JSON API)
- Structured audit logging
- Starlette/FastAPI middleware and `require_permission` dependency

## Usage

```python
from eel_auth import JwksCache, SubjectExtractor, AuthMiddleware

cache = JwksCache("https://keycloak.example.com/realms/myrealm/protocol/openid-connect/certs")
extractor = SubjectExtractor(realm="myrealm")

app.add_middleware(AuthMiddleware, jwks_cache=cache, extractor=extractor)
```
