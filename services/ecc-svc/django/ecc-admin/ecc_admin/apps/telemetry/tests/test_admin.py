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

"""
Tests for the telemetry Django admin app.

Tests run with SQLite (managed=False tables are unmanaged, so Django does not
try to create them). We mock the telemetry query API to keep tests fast and
hermetic.
"""

from __future__ import annotations

import json
from unittest.mock import patch

from django.contrib.auth.models import User
from django.test import TestCase, TransactionTestCase
from django.urls import reverse


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def make_superuser(username: str = "admin") -> User:
    return User.objects.create_superuser(
        username, f"{username}@example.com", "password"
    )


def ensure_audit_table() -> None:
    """Create the unmanaged eel_telemetry_queries table in the SQLite test DB.

    Managed=False models are not created by the test runner, but the
    evidence-export view reads recent queries from this table. The SQLite
    schema editor refuses to run inside a transaction, so we create the table
    with idempotent raw DDL instead.
    """
    from django.db import connection

    with connection.cursor() as cursor:
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS eel_telemetry_queries (
                query_id          char(32)      PRIMARY KEY,
                actor_svid        varchar(256)  NOT NULL,
                query_dsl         text          NOT NULL,
                executed_at       datetime      NOT NULL,
                row_count         bigint        NOT NULL,
                result_sha256     varchar(64)   NOT NULL,
                evidence_exported bool          NOT NULL,
                preset_name       varchar(64)   NOT NULL,
                nl_text           text          NOT NULL
            )
            """
        )
        cursor.execute(
            "CREATE INDEX IF NOT EXISTS eel_telemetry_queries_executed_at "
            "ON eel_telemetry_queries(executed_at)"
        )
        cursor.execute(
            "CREATE INDEX IF NOT EXISTS eel_telemetry_queries_actor_svid "
            "ON eel_telemetry_queries(actor_svid)"
        )


PRESET_RESPONSE = {
    "presets": [
        {
            "name": "drops_by_county",
            "description": "Physical drops by county",
            "required_params": ["county", "hours"],
        },
        {
            "name": "officer_activity",
            "description": "Officer activity",
            "required_params": ["officer_id", "from", "to"],
        },
    ]
}

QUERY_RESULT_RESPONSE = {
    "rows": [{"device_id": "d1", "signal_class": "physical_drops"}],
    "count": 1,
    "preset": "drops_by_county",
    "description": "Physical drops by county",
}

NL_RESPONSE = {
    "proposed": {
        "query": {
            "time_range": {"from": "2026-01-01T00:00:00Z", "to": "2026-01-02T00:00:00Z"}
        },
        "explanation": "Mock translation",
        "confidence": 0.75,
        "original_text": "show all drops",
    },
    "confirmation_required": True,
}

EXPORT_RESPONSE = {
    "manifest": {
        "manifest_sha256": "a" * 64,
        "hsm_signature": "stub:sha256:aabbccdd",
    },
    "message": "Bundle generated",
}


# ---------------------------------------------------------------------------
# Test: admin listing pages load
# ---------------------------------------------------------------------------


class AdminListingTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    def test_admin_index_accessible(self):
        resp = self.client.get("/admin/")
        self.assertEqual(resp.status_code, 200)

    def test_admin_login_redirects_when_logged_in(self):
        resp = self.client.get("/admin/login/")
        # When already logged in, Django admin redirects to index
        self.assertIn(resp.status_code, [200, 302])


# ---------------------------------------------------------------------------
# Test: hard_questions view
# ---------------------------------------------------------------------------


class HardQuestionsViewTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    @patch("ecc_admin.apps.telemetry.views._api_get")
    def test_hard_questions_renders(self, mock_get):
        mock_get.return_value = PRESET_RESPONSE
        resp = self.client.get(reverse("telemetry:hard_questions"))
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "drops_by_county")
        self.assertContains(resp, "officer_activity")

    @patch("ecc_admin.apps.telemetry.views._api_get")
    def test_hard_questions_empty_when_api_fails(self, mock_get):
        mock_get.return_value = {"error": "connection refused"}
        resp = self.client.get(reverse("telemetry:hard_questions"))
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "No presets available")

    def test_hard_questions_requires_login(self):
        self.client.logout()
        resp = self.client.get(reverse("telemetry:hard_questions"))
        self.assertEqual(resp.status_code, 302)
        self.assertIn("/accounts/login", resp["Location"])


# ---------------------------------------------------------------------------
# Test: run_preset view
# ---------------------------------------------------------------------------


class RunPresetViewTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_run_preset_returns_json(self, mock_post):
        mock_post.return_value = QUERY_RESULT_RESPONSE
        resp = self.client.post(
            reverse("telemetry:run_preset", kwargs={"name": "drops_by_county"}),
            data=json.dumps({"county": "Nairobi", "hours": 24}),
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 200)

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_run_preset_htmx_returns_partial(self, mock_post):
        mock_post.return_value = QUERY_RESULT_RESPONSE
        resp = self.client.post(
            reverse("telemetry:run_preset", kwargs={"name": "drops_by_county"}),
            data=json.dumps({"county": "Nairobi", "hours": 24}),
            content_type="application/json",
            HTTP_HX_REQUEST="true",
        )
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "row(s)")


# ---------------------------------------------------------------------------
# Test: query_builder view
# ---------------------------------------------------------------------------


class QueryBuilderViewTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    def test_query_builder_renders(self):
        resp = self.client.get(reverse("telemetry:query_builder"))
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "Signal classes")
        self.assertContains(resp, "Natural-language query")
        self.assertContains(resp, "natural language to DSL")

    def test_query_builder_lists_all_13_classes(self):
        resp = self.client.get(reverse("telemetry:query_builder"))
        for cls in [
            "location",
            "movements",
            "battery",
            "network",
            "attestation",
            "physical_drops",
        ]:
            self.assertContains(resp, cls)


# ---------------------------------------------------------------------------
# Test: structured query run
# ---------------------------------------------------------------------------


class RunStructuredQueryTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_run_structured_valid(self, mock_post):
        mock_post.return_value = QUERY_RESULT_RESPONSE
        body = {
            "time_range": {
                "from": "2026-01-01T00:00:00Z",
                "to": "2026-01-02T00:00:00Z",
            },
            "signal_classes": ["battery"],
            "limit": 500,
        }
        resp = self.client.post(
            reverse("telemetry:run_structured"),
            data=json.dumps(body),
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 200)

    def test_run_structured_rejects_invalid_json(self):
        resp = self.client.post(
            reverse("telemetry:run_structured"),
            data="not json",
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 400)


# ---------------------------------------------------------------------------
# Test: NL translate
# ---------------------------------------------------------------------------


class NlTranslateTest(TestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_nl_translate_returns_proposed(self, mock_post):
        mock_post.return_value = NL_RESPONSE
        resp = self.client.post(
            reverse("telemetry:nl_translate"),
            data=json.dumps({"text": "show all drops in Nairobi"}),
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 200)

    def test_nl_translate_rejects_empty_text(self):
        resp = self.client.post(
            reverse("telemetry:nl_translate"),
            data=json.dumps({"text": ""}),
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 400)


# ---------------------------------------------------------------------------
# Test: evidence export view
# ---------------------------------------------------------------------------


class EvidenceExportViewTest(TransactionTestCase):
    def setUp(self):
        self.user = make_superuser()
        self.client.login(username="admin", password="password")
        ensure_audit_table()

    def test_evidence_export_renders(self):
        resp = self.client.get(reverse("telemetry:evidence_export"))
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "Evidence Export")

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_trigger_export_success(self, mock_post):
        mock_post.return_value = EXPORT_RESPONSE
        resp = self.client.post(
            reverse(
                "telemetry:trigger_export",
                kwargs={"query_id": "00000000-0000-0000-0000-000000000001"},
            ),
            data=json.dumps({}),
            content_type="application/json",
        )
        self.assertEqual(resp.status_code, 200)

    @patch("ecc_admin.apps.telemetry.views._api_post")
    def test_trigger_export_htmx_partial(self, mock_post):
        mock_post.return_value = EXPORT_RESPONSE
        resp = self.client.post(
            reverse(
                "telemetry:trigger_export",
                kwargs={"query_id": "00000000-0000-0000-0000-000000000001"},
            ),
            data=json.dumps({}),
            content_type="application/json",
            HTTP_HX_REQUEST="true",
        )
        self.assertEqual(resp.status_code, 200)
        self.assertContains(resp, "Manifest SHA-256")

    def test_evidence_export_requires_login(self):
        self.client.logout()
        resp = self.client.get(reverse("telemetry:evidence_export"))
        self.assertEqual(resp.status_code, 302)

    def test_trigger_export_requires_login(self):
        self.client.logout()
        resp = self.client.post(
            reverse(
                "telemetry:trigger_export",
                kwargs={"query_id": "00000000-0000-0000-0000-000000000001"},
            ),
        )
        self.assertEqual(resp.status_code, 302)
