# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Tests for authoring views using Django test client."""

import uuid
from unittest.mock import MagicMock, patch

import pytest
from django.test import Client, RequestFactory
from django.urls import reverse


pytestmark = pytest.mark.django_db


@pytest.fixture
def client():
    return Client()


@pytest.fixture
def factory():
    return RequestFactory()


def test_report_list_ok(client):
    resp = client.get(reverse("report_list"))
    assert resp.status_code == 200
    assert b"Ad-hoc Reports" in resp.content


def test_report_list_with_scope_filter(client):
    resp = client.get(reverse("report_list") + "?scope=shared")
    assert resp.status_code == 200


def test_report_list_with_search(client):
    resp = client.get(reverse("report_list") + "?q=test")
    assert resp.status_code == 200


def test_report_editor_unknown_id(client):
    fake_id = str(uuid.uuid4())
    resp = client.get(reverse("report_editor", args=[fake_id]))
    assert resp.status_code == 200
    # Should render even without a report (dev mode)
    assert b"Editor" in resp.content or b"editor" in resp.content.lower()


def test_htmx_save_version_returns_lint(client):
    fake_id = str(uuid.uuid4())
    mock_response = MagicMock()
    mock_response.status_code = 201
    mock_response.json.return_value = {
        "lint_report": {"errors": [], "warnings": []},
        "safety_score": 0.9,
    }
    mock_response.raise_for_status = MagicMock()

    with patch("adhoc_editor.apps.authoring.views.httpx.Client") as MockClient:
        mock_ctx = MockClient.return_value.__enter__.return_value
        mock_ctx.post.return_value = mock_response
        resp = client.post(
            reverse("htmx_save_version", args=[fake_id]),
            {"source": "SELECT * FROM eel_voters LIMIT 10", "source_type": "sql"},
        )

    assert resp.status_code == 200
    assert (
        b"safety" in resp.content.lower()
        or b"score" in resp.content.lower()
        or b"issues" in resp.content.lower()
    )


def test_htmx_save_version_api_error_graceful(client):
    fake_id = str(uuid.uuid4())
    with patch("adhoc_editor.apps.authoring.views.httpx.Client") as MockClient:
        mock_ctx = MockClient.return_value.__enter__.return_value
        mock_ctx.post.side_effect = Exception("connection refused")
        resp = client.post(
            reverse("htmx_save_version", args=[fake_id]),
            {"source": "SELECT 1", "source_type": "sql"},
        )
    assert resp.status_code == 200
    assert b"error" in resp.content.lower() or b"API error" in resp.content


def test_htmx_preview_returns_table(client):
    fake_version_id = str(uuid.uuid4())
    mock_response = MagicMock()
    mock_response.status_code = 201
    mock_response.json.return_value = {
        "preview_id": str(uuid.uuid4()),
        "rows": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}],
    }
    mock_response.raise_for_status = MagicMock()

    with patch("adhoc_editor.apps.authoring.views.httpx.Client") as MockClient:
        mock_ctx = MockClient.return_value.__enter__.return_value
        mock_ctx.post.return_value = mock_response
        resp = client.post(
            reverse("htmx_preview", args=[fake_version_id]),
            {"format": "json", "sample_limit": "10"},
        )
    assert resp.status_code == 200
    assert b"Alice" in resp.content or b"rows" in resp.content.lower()


def test_publish_wizard_get(client):
    fake_version_id = str(uuid.uuid4())
    resp = client.get(reverse("publish_wizard", args=[fake_version_id]))
    assert resp.status_code == 200
    assert b"Publish" in resp.content


def test_publish_wizard_post_creates_definition(client):
    fake_version_id = str(uuid.uuid4())
    def_id = str(uuid.uuid4())
    mock_response = MagicMock()
    mock_response.status_code = 201
    mock_response.json.return_value = {"definition_id": def_id}
    mock_response.raise_for_status = MagicMock()

    with patch("adhoc_editor.apps.authoring.views.httpx.Client") as MockClient:
        mock_ctx = MockClient.return_value.__enter__.return_value
        mock_ctx.post.return_value = mock_response
        resp = client.post(
            reverse("publish_wizard", args=[fake_version_id]),
            {"formats": ["docx", "csv"]},
        )
    assert resp.status_code == 200
    assert def_id.encode() in resp.content


def test_publish_wizard_post_error_graceful(client):
    fake_version_id = str(uuid.uuid4())
    with patch("adhoc_editor.apps.authoring.views.httpx.Client") as MockClient:
        mock_ctx = MockClient.return_value.__enter__.return_value
        mock_ctx.post.side_effect = Exception("network error")
        resp = client.post(
            reverse("publish_wizard", args=[fake_version_id]),
            {"formats": ["docx"]},
        )
    assert resp.status_code == 200
    assert b"error" in resp.content.lower() or b"failed" in resp.content.lower()


def test_admin_registered_models(client):
    from django.contrib import admin as dj_admin
    from adhoc_editor.apps.authoring.models import (
        AdhocReport,
        AdhocReportVersion,
        AdhocPreview,
        AdhocShare,
        AdhocSnippet,
    )

    for model in [
        AdhocReport,
        AdhocReportVersion,
        AdhocPreview,
        AdhocShare,
        AdhocSnippet,
    ]:
        assert model in dj_admin.site._registry, f"{model} not registered with admin"


def test_report_editor_highlight_failure_is_logged(client, caplog):
    import logging

    fake_id = str(uuid.uuid4())
    fake_version = MagicMock()
    fake_version.source = "SELECT 1"
    fake_report = MagicMock()
    fake_report.versions.order_by.return_value.first.return_value = fake_version

    with (
        patch("adhoc_editor.apps.authoring.models.AdhocReport") as MockReport,
        patch(
            "adhoc_editor.apps.authoring.views.sqlparse.format",
            side_effect=ValueError("bad sql"),
        ),
    ):
        MockReport.objects.get.return_value = fake_report
        with caplog.at_level(logging.WARNING):
            resp = client.get(reverse("report_editor", args=[fake_id]))

    assert resp.status_code == 200
    assert any(
        "highlight failed" in record.message for record in caplog.records
    )
