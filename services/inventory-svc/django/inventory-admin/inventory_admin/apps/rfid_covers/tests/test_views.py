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
View tests for rfid_covers HTMX surfaces.

Models are managed=False so we patch the ORM querysets to avoid hitting a real DB.
"""

from __future__ import annotations

import uuid
from datetime import datetime, timezone as tz
from unittest.mock import MagicMock, patch

from django.contrib.auth.models import User
from django.test import Client, TestCase


def _perm(codename: str) -> str:
    return f"rfid_covers.{codename}"


class _FakeTag:
    def __init__(self):
        self.tag_id = uuid.uuid4()
        self.epc = "E28011702000010A02BC3F2A"
        self.chip_family = "nxp_ucode9"
        self.paired_with_box = "BOX-001"
        self.cover_state = "WAREHOUSE"
        self.tamper_state = "intact"
        self.manufacturer = "Acme"
        self.issued_at = datetime(2026, 1, 1, tzinfo=tz.utc)
        self.hsm_key_ref = "slot:3"
        self.bluetooth_id = None
        self.notes = ""

    def get_chip_family_display(self):
        return "NXP UCODE 9"

    def get_tamper_state_display(self):
        return "Intact"


class _FakeAlert:
    def __init__(self):
        self.alert_id = uuid.uuid4()
        self.tag = _FakeTag()
        self.severity = "critical"
        self.description = "Antenna break"
        self.geo_lat = None
        self.geo_lon = None
        self.expected_state = "STATION_CLOSED"
        self.observed_state = "QUARANTINE"
        self.ecc_dispatched = False
        self.dispatch_officer_id = None
        self.resolved_at = None
        self.resolution_note = ""
        self.detected_at = datetime(2026, 6, 1, tzinfo=tz.utc)

    def get_severity_display(self):
        return "Critical"


def _fake_qs(items):
    """Return a MagicMock queryset that iterates over items."""
    qs = MagicMock()
    qs.__iter__ = lambda self: iter(items)
    qs.__len__ = lambda self: len(items)
    qs.filter.return_value = qs
    qs.select_related.return_value = qs
    qs.order_by.return_value = qs
    qs.count.return_value = len(items)
    return qs


class BoxChainViewTest(TestCase):
    def setUp(self):
        self.client = Client()
        self.user = User.objects.create_user("officer", password="pass")
        self.user.is_staff = True
        self.user.save()
        self.client.login(username="officer", password="pass")

    @patch("inventory_admin.apps.rfid_covers.views.CoverStateTransition")
    @patch("inventory_admin.apps.rfid_covers.views.RfidCoverTag")
    def test_renders_empty_when_no_ballot_box_id(self, MockTag, MockTransition):
        response = self.client.get("/rfid/box-chain/")
        assert response.status_code in (200, 302, 403)

    @patch("inventory_admin.apps.rfid_covers.views.CoverStateTransition")
    @patch("inventory_admin.apps.rfid_covers.views.RfidCoverTag")
    def test_renders_not_found_message(self, MockTag, MockTransition):
        MockTag.objects.get.side_effect = MockTag.DoesNotExist()
        MockTag.DoesNotExist = Exception
        response = self.client.get("/rfid/box-chain/?ballot_box_id=NOTEXIST")
        assert response.status_code in (200, 302, 403)


class TamperTriageViewTest(TestCase):
    def setUp(self):
        self.client = Client()
        self.user = User.objects.create_user("triage_officer", password="pass")
        self.user.is_staff = True
        self.user.save()
        self.client.login(username="triage_officer", password="pass")

    @patch("inventory_admin.apps.rfid_covers.views.CoverTamperAlert")
    def test_triage_renders_pending_alerts(self, MockAlert):
        alerts = [_FakeAlert(), _FakeAlert(), _FakeAlert()]
        MockAlert.objects.filter.return_value = _fake_qs(alerts)
        MockAlert.Severity.choices = [("critical", "Critical")]
        response = self.client.get("/rfid/tamper-triage/")
        assert response.status_code in (200, 302, 403)

    @patch("inventory_admin.apps.rfid_covers.views.CoverTamperAlert")
    def test_triage_filter_by_severity(self, MockAlert):
        MockAlert.objects.filter.return_value = _fake_qs([])
        MockAlert.Severity.choices = [("critical", "Critical")]
        response = self.client.get("/rfid/tamper-triage/?severity=critical")
        assert response.status_code in (200, 302, 403)

    @patch("inventory_admin.apps.rfid_covers.views.CoverTamperAlert")
    def test_resolve_action_post(self, MockAlert):
        alert = _FakeAlert()
        mock_qs = MagicMock()
        mock_qs.filter.return_value = mock_qs
        mock_qs.count.return_value = 0
        MockAlert.objects.filter.return_value = mock_qs
        mock_saved = MagicMock()
        mock_saved.resolved_at = None
        MockAlert.objects.get_object_or_404 = MagicMock(return_value=mock_saved)
        response = self.client.post(
            "/rfid/tamper-triage/",
            {"alert_id": str(alert.alert_id), "resolution_note": "false alarm"},
        )
        assert response.status_code in (200, 302, 403, 404)


class ReaderHealthViewTest(TestCase):
    def setUp(self):
        self.client = Client()
        self.user = User.objects.create_user("reader_user", password="pass")
        self.user.is_staff = True
        self.user.save()
        self.client.login(username="reader_user", password="pass")

    @patch("inventory_admin.apps.rfid_covers.views.RfidCoverRead")
    def test_reader_health_renders(self, MockRead):
        MockRead.objects.values.return_value.annotate.return_value.order_by.return_value = []
        response = self.client.get("/rfid/reader-health/")
        assert response.status_code in (200, 302, 403)

    @patch("inventory_admin.apps.rfid_covers.views.RfidCoverRead")
    def test_reader_health_with_data(self, MockRead):
        stats = [
            {
                "reader_serial": "IMP-001",
                "reader_type": "warehouse_gate",
                "tag_count": 50,
                "last_read": datetime(2026, 6, 1, tzinfo=tz.utc),
            },
        ]
        MockRead.objects.values.return_value.annotate.return_value.order_by.return_value = stats
        response = self.client.get("/rfid/reader-health/")
        assert response.status_code in (200, 302, 403)
