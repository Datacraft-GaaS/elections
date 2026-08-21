# {{cookiecutter.svc_title}}

{{cookiecutter.svc_description}}

## Overview

Service: `{{cookiecutter.svc_name}}`
Port: `{{cookiecutter.port}}`
Data class: `{{cookiecutter.data_class}}`
Election context: `{{cookiecutter.election_id}}`

## Running locally

```bash
cd py
uv sync
uv run uvicorn {{cookiecutter.svc_name}}_api.main:app --reload
```
