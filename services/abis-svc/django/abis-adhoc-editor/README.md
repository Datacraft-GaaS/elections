# ABIS Ad Hoc Editor

Django admin app for composing custom biometric reports. Analysts build ad-hoc SQL report templates, preview HTML/PDF output, and schedule batch exports via Kafka events.

## Overview

Module: M03 (ABIS - Automated Biometric Identification System)

The ABIS ad-hoc editor powers the Astro adhoc-editor frontend, enabling rapid ad-hoc reporting without backend code changes. Templates are versioned, sandboxed (read-only SQL), and previewed before export.

## Key Features

- **Report Template CRUD**: Compose SQL queries with Jinja2 templating over whitelisted tables
- **Preview Rendering**: HTML/PDF output preview before scheduling
- **Scheduled Exports**: Set batch export jobs with date/time trigger or one-off immediate export
- **Audit Trail**: TemplateIndex, MatchRequest stats (managed=False, read-only), AuditEvent logging
- **Admin Views**: ReportTemplateAdmin, ScheduledExportAdmin, BatchResultAdmin

## HTMX Endpoints

- POST/GET `/abis/adhoc/preview/{id}` - Render template preview
- POST/GET `/abis/adhoc/export/{id}` - Trigger or retrieve batch export

## Models (Managed=False)

- TemplateIndex - Report template versions
- MatchRequest - Biometric match statistics (read-only)
- AuditEvent - Template composition audit trail

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `ABIS_API_URL` - ABIS matcher gRPC endpoint
- `ABIS_ADHOC_ALLOWED_GROUPS` - Comma-separated SpiceDB roles (ABIS_ANALYST, ABIS_SUPERVISOR)

## Running Locally

```bash
uv sync
uv run manage.py runserver
```

## Tests

```bash
uv run pytest -vxs tests/ci
```

## Deployment

WCAG 2.2 AA compliant. Pod resource: CPU 500m, Memory 512Mi. 

Kubernetes manifest: `services/abis-svc/deploy/base/adhoc-editor-deployment.yaml`

Production run:

```bash
gunicorn abis_adhoc_editor.wsgi:application
```

## Permissions

ABIS_ANALYST, ABIS_SUPERVISOR roles enforced via SpiceDB. Queries are sandboxed to prevent destructive SQL.

## Notes

- Templates do not support INSERT/UPDATE/DELETE (SELECT-only validation)
- PDF rendering uses WeasyPrint (install wkhtmltopdf for system fonts)
- Large result sets (>100k rows) may timeout; use pagination
