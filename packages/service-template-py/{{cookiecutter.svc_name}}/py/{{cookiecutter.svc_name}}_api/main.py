"""{{cookiecutter.svc_title}} API entry point."""

from fastapi import FastAPI

from .eel_server import create_app
from .routers import health

app: FastAPI = create_app(title="{{cookiecutter.svc_title}}")
app.include_router(health.router)
