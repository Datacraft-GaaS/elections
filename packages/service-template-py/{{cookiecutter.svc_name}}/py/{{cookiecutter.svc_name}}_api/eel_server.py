"""FastAPI wiring: auth + resilience + observability."""

import os

from fastapi import FastAPI
from starlette.middleware.cors import CORSMiddleware

from eel_auth import AuthMiddleware, JwksCache, SubjectExtractor
from eel_loadshedding import LoadSheddingMiddleware
from eel_observability import init_otel
from eel_resilience import IdempotencyMiddleware, InMemoryIdempotencyStore

# Paths that must stay reachable without a Bearer token.
_PUBLIC_PATHS = ["/health/live", "/health/ready"]

_DEFAULT_CORS_ORIGIN = "http://localhost:{{cookiecutter.port}}"


def _jwks_url() -> str:
    explicit = os.environ.get("EEL_JWKS_URL")
    if explicit:
        return explicit
    base = os.environ.get("EEL_KEYCLOAK_URL", "http://localhost:8080")
    realm = os.environ.get("EEL_KEYCLOAK_REALM", "eel")
    return f"{base}/realms/{realm}/protocol/openid-connect/certs"


def _cors_origins() -> list[str]:
    raw = os.environ.get("EEL_CORS_ORIGINS", "")
    origins = [origin.strip() for origin in raw.split(",") if origin.strip()]
    return origins or [_DEFAULT_CORS_ORIGIN]


def create_app(
    *,
    title: str = "{{cookiecutter.svc_title}}",
    debug: bool = False,
) -> FastAPI:
    """Build the FastAPI app with the standard eel middleware stack."""
    app = FastAPI(title=title, debug=debug)

    init_otel(
        os.environ.get("EEL_OTEL_SERVICE_NAME", "{{cookiecutter.svc_name}}"),
        otlp_endpoint=os.environ.get("EEL_OTLP_ENDPOINT", "http://localhost:4317"),
    )

    # Request flow is CORS -> load shedding -> idempotency -> auth -> routes.
    # Starlette runs the last-added middleware first, so registration order is
    # reversed.
    app.add_middleware(
        AuthMiddleware,
        JwksCache(jwks_url=_jwks_url()),
        SubjectExtractor(realm=os.environ.get("EEL_KEYCLOAK_REALM", "eel")),
        exclude_paths=_PUBLIC_PATHS,
        audience=os.environ.get("EEL_JWT_AUDIENCE") or None,
        issuer=os.environ.get("EEL_JWT_ISSUER") or None,
    )
    app.add_middleware(IdempotencyMiddleware, InMemoryIdempotencyStore())
    app.add_middleware(LoadSheddingMiddleware)
    app.add_middleware(
        CORSMiddleware,
        allow_origins=_cors_origins(),
        allow_methods=["*"],
        allow_headers=["*"],
    )
    return app
