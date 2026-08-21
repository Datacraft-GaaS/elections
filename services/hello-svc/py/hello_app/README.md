# Hello FastAPI App

Hello-world FastAPI app template for scaffolding new election services. Not production.

## Overview

Sample FastAPI service used as template for service initialization and CI smoke tests.

## Endpoints

- GET `/` - Hello message
- GET `/health` - Health check

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Run

```bash
gunicorn hello_app.wsgi:application
```
