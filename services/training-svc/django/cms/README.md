# Voter Education CMS Admin

Django admin console for voter education content management. Manages content items, versioned revisions, fact-check reviews, multilingual content (11 languages), and WhatsApp/CDN distribution scheduling.

## Overview

Module: M37 (Voter Education CMS)

Content management system for producing multilingual voter education materials covering voter rights, polling procedures, candidate/party information, and accessibility guidance. Content is distributed via WhatsApp, SMS, and the citizen-facing portal.

## Key Features

- **Content Management**: Create, edit, and publish voter education articles/videos
- **Version Control**: Track content revisions with publication dates
- **Fact-Checking**: Submit content for fact-check review with approval workflow
- **Multilingual Support**: Translate content to 11 Kenyan/regional languages via NLLB
- **WhatsApp Distribution**: Schedule content delivery to subscribed voters via WhatsApp bot
- **CDN Publishing**: Stage content for public portal with caching headers

## HTMX Endpoints

- GET `/cms/admin/content/{id}/preview` - Preview rendered content in all languages
- POST `/cms/admin/content/{id}/publish` - Publish content to CDN and WhatsApp channels

## Models (Managed=False)

- ContentItem - Voter education content pieces
- ContentVersion - Versioned revisions with publication timestamps
- FactCheckReview - Fact-check approval records

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `NLLB_ADAPTER_URL` - Translation service (NLLB 200 model)
- `WA_ADAPTER_URL` - WhatsApp API endpoint for distribution
- `CDN_UPLOAD_URL` - CDN endpoint for portal content

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

Kubernetes manifest: `services/training-svc/deploy/base/cms-deployment.yaml`

Production run:

```bash
gunicorn cms.wsgi:application
```

## Permissions

CMS_EDITOR, CMS_SUPERVISOR, FACT_CHECKER roles via SpiceDB. Editors create content; supervisors approve publishing; fact-checkers verify accuracy.

## Notes

- All content is translated to 11 languages (Swahili, English, plus 9 regional languages)
- Fact-check review is mandatory before publication
- WhatsApp templates require 6-8 week approval lead time with Meta
- Content is publicly archived for post-election research (delay-published)
