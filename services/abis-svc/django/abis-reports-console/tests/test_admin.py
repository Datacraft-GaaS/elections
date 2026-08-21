"""Admin tests for abis-reports-console."""

import uuid
from datetime import datetime, timezone

import pytest
from django.contrib.auth import get_user_model
from django.test import Client

User = get_user_model()


def _superuser():
    return User.objects.create_superuser("admin", "admin@test.local", "pass")


@pytest.mark.django_db
def test_definition_admin_list_renders(client, definition):
    admin = _superuser()
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportdefinition/")
    assert resp.status_code == 200
    assert definition.slug.encode() in resp.content


@pytest.mark.django_db
def test_run_admin_list_renders(client, run):
    admin = _superuser()
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportrun/")
    assert resp.status_code == 200
    assert run.slug.encode() in resp.content


@pytest.mark.django_db
def test_run_admin_filter_by_status(client, run):
    admin = _superuser()
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportrun/?status=complete")
    assert resp.status_code == 200


@pytest.mark.django_db
def test_artifact_admin_list_renders(client, artifact):
    admin = _superuser()
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportartifact/")
    assert resp.status_code == 200


@pytest.mark.django_db
def test_run_now_action_creates_queued_run(client, definition):
    admin = _superuser()
    client.force_login(admin)
    resp = client.post(
        "/admin/eel_reports/reportdefinition/",
        {
            "action": "run_now",
            "_selected_action": [str(definition.definition_id)],
        },
    )
    assert resp.status_code in (200, 302)

    from abis_reports_console.apps.reports.models import ReportRun

    run = ReportRun.objects.filter(slug=definition.slug, status="queued").first()
    assert run is not None


@pytest.mark.django_db
def test_access_log_admin_is_readonly(client, artifact):
    admin = _superuser()
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportaccesslog/add/")
    # Add not permitted for access log.
    assert resp.status_code == 403
