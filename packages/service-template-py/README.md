# service-template-py

FastAPI service cookiecutter template for eel Python services: generates the standard async service layout pre-wired with all `eel-*-py` packages.

## Purpose in context

New Python services in `services/<svc>/py/` are scaffolded from this template. The generated project uses FastAPI with async SQLAlchemy, Pydantic v2, tab indentation, and all shared eel middleware already wired. It includes a conformance test module that any service can run to verify it meets the API conventions defined in ADR-0011.

## Generated layout

```
<svc>/
  src/<pkg>/
    main.py        # FastAPI app factory
    router.py      # domain routes
    models.py      # SQLAlchemy ORM models
    views.py       # Pydantic request/response schemas
    service.py     # domain logic
    worker.py      # background Kafka consumer (optional)
  tests/
    ci/            # pytest-discoverable CI tests
    conformance/   # ADR-0011 conformance suite
  pyproject.toml
  Dockerfile
```

## Usage

```bash
# Scaffold a new Python service:
uv run cookiecutter packages/service-template-py/template \
    --output-dir services/my-svc/py
```

Prompts: `service_name`, `package_name`, `enable_kafka_consumer` (y/n).

## Conformance suite

```python
# tests/conformance/test_conformance.py (generated)
import pytest
from eel_service_template.conformance import run_all

@pytest.mark.asyncio
async def test_api_conformance():
    await run_all("http://localhost:8080")
```

## Dependencies

`cookiecutter>=2.5`. Generated project depends on `fastapi`, `uvicorn`, `sqlalchemy`, and all `eel-*-py` packages from this workspace.

## Install

```bash
uv add eel-service-template
```

## Test (the template itself)

```bash
uv run pytest packages/service-template-py/tests -v
```

## Related docs

- `packages/service-template-rs/README.md` - Rust counterpart
- `docs/plans/00-principles.md` - standard service layout conventions
- `docs/adr/0011-api-conventions.md` - what the conformance suite verifies

## Limitations

The template requires `cookiecutter` CLI. The generated `worker.py` stub requires `aiokafka`; add it to the generated `pyproject.toml` if the service consumes Kafka topics.
