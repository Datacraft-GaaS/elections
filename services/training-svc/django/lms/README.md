# Poll Worker Training LMS Admin

Django admin console for poll worker training and certification. Manages course modules, worker enrollments, completion tracking, certificate generation, and offline bundle export for the Flutter LMS offline app.

## Overview

Module: M38 (Poll Worker Training LMS)

Learning management system for training and certifying election staff (poll workers, supervisors, observers). Content can be completed online or via offline bundle for low-connectivity areas.

## Key Features

- **Course Management**: Create, organize, and publish training course modules
- **Worker Enrollment**: Register workers in courses with bulk enrollment support
- **Progress Tracking**: Monitor completion status and assessment scores
- **Certificate Generation**: Issue signed digital certificates upon course completion
- **Offline Bundle Export**: Package courses for offline delivery via Flutter mobile app
- **Admin Views**: CourseModuleAdmin, EnrollmentAdmin (bulk enroll), CertificateAdmin

## HTMX Endpoints

- GET `/lms/admin/course/{id}/modules` - List course modules and resources
- POST `/lms/admin/enrollment/{id}/certificate` - Generate and download completion certificate

## Models (Managed=False)

- CourseModule - Training course content modules
- WorkerEnrollment - Worker registration in courses
- WorkerCertificate - Completion certificates

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `CERT_SIGN_KEY` - Private key for certificate signing (PEM format)
- `OFFLINE_BUNDLE_EXPORT_PATH` - S3 or local directory for offline bundles

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

WCAG 2.2 AA compliant. Pod resource: CPU 250m, Memory 256Mi.

Kubernetes manifest: `services/training-svc/deploy/base/lms-deployment.yaml`

Production run:

```bash
gunicorn lms.wsgi:application
```

## Permissions

LMS_ADMIN, LMS_INSTRUCTOR roles via SpiceDB. Admins manage courses and enrollments; instructors create/update content.

## Notes

- Certificates are digitally signed and verified at polling stations
- Offline bundles are encrypted for low-connectivity field deployment
- Course completion is tracked against poll worker roster for validation
- Certificates are issued immediately upon passing final assessment (no delay)
