"""FastAPI wiring: auth + resilience + observability + outbox."""

from fastapi import FastAPI
from starlette.middleware.cors import CORSMiddleware


def create_app(
    *,
    title: str = "{{cookiecutter.svc_title}}",
    debug: bool = False,
) -> FastAPI:
    app = FastAPI(title=title, debug=debug)
    # TODO: wire EelAuthMiddleware, IdempotencyMiddleware, LoadSheddingMiddleware
    # TODO: wire OTel via init_otel()
    return app
