# eel-service-template

Cookiecutter template for FastAPI services on the eel platform.

## Usage

```bash
uvx cookiecutter gh:eel/service-template-py
# or locally:
uvx cookiecutter packages/service-template-py
```

## Prompts

| Variable | Default | Description |
|---|---|---|
| `svc_name` | `my_service` | Python package name (snake_case) |
| `svc_title` | `My Service` | Human-readable title |
| `svc_description` | `A new eel service` | Short description |
| `author` | `eel-team` | Author |
| `python_version` | `3.12` | Python version |
| `port` | `8000` | Container port |
| `data_class` | `B` | PII data classification tier |
| `election_id` | `2027-general` | Election context |

## Generated structure

```
<svc_name>/
  py/
    <svc_name>_api/
      main.py
      eel_server.py
      routers/
        health.py
    pyproject.toml
  Dockerfile
  deploy/service.yaml
  docs/
    README.md  EVENTS.md  THREAT-MODEL.md  SLA.md
  .woodpecker/<svc_name>.yaml
```
