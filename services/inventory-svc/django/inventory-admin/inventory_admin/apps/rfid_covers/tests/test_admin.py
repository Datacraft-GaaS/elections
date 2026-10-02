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
Admin smoke tests for rfid_covers.

Uses Django's test client authenticated as a superuser.
All models are managed=False, so we substitute SQLite in-memory tables via
a fake migration (CreateModel in test setup) or bypass the DB by mocking querysets.
"""

from __future__ import annotations

import uuid
from datetime import timezone as tz
from unittest.mock import MagicMock, patch

import pytest
from django.contrib.admin.sites import AdminSite
from django.contrib.auth.models import User
from django.test import RequestFactory, TestCase

from inventory_admin.apps.rfid_covers.admin import (
    CoverStateTransitionAdmin,
    CoverTamperAlertAdmin,
    RfidCoverReadAdmin,
    RfidCoverTagAdmin,
)
from inventory_admin.apps.rfid_covers.models import CoverTamperAlert, RfidCoverTag


class _Ns:
    """Namespace helper for building fake model objects."""

    def __init__(self, **kw):
        for k, v in kw.items():
            setattr(self, k, v)


def _make_tag(**kw) -> _Ns:
    defaults = dict(
        tag_id=uuid.uuid4(),
        epc="E28011702000010A02BC3F2A",
        chip_family="nxp_ucode9",
        paired_with_box="BOX-001",
        tamper_state="intact",
        cover_state="WAREHOUSE",
        manufacturer="Acme RFID",
        issued_at=__import__("datetime").datetime(2026, 1, 1, tzinfo=tz.utc),
        hsm_key_ref="slot:3",
        bluetooth_id=None,
        notes="",
    )
    defaults.update(kw)
    return _Ns(**defaults)


def _make_alert(**kw) -> _Ns:
    defaults = dict(
        alert_id=uuid.uuid4(),
        tag_id=uuid.uuid4(),
        severity="critical",
        description="Cover lift detected",
        geo_lat=None,
        geo_lon=None,
        expected_state="STATION_CLOSED",
        observed_state="QUARANTINE",
        ecc_dispatched=False,
        dispatch_officer_id=None,
        resolved_at=None,
        resolution_note="",
        detected_at=__import__("datetime").datetime(2026, 6, 1, tzinfo=tz.utc),
        is_resolved=False,
    )
    defaults.update(kw)
    return _Ns(**defaults)


@pytest.fixture()
def superuser(db):
    return User.objects.create_superuser("admin", "admin@example.com", "password")


@pytest.fixture()
def rf():
    return RequestFactory()


class TestRfidCoverTagAdminListDisplay(TestCase):
    def test_list_display_fields(self):
        admin_obj = RfidCoverTagAdmin(RfidCoverTag, AdminSite())
        assert "epc" in admin_obj.list_display
        assert "manufacturer" in admin_obj.list_display
        assert "paired_with_box" in admin_obj.list_display
        assert "tamper_state" in admin_obj.list_display
        assert "issued_at" in admin_obj.list_display

    def test_list_filter_fields(self):
        admin_obj = RfidCoverTagAdmin(RfidCoverTag, AdminSite())
        assert "tamper_state" in admin_obj.list_filter
        assert "chip_family" in admin_obj.list_filter

    def test_search_fields(self):
        admin_obj = RfidCoverTagAdmin(RfidCoverTag, AdminSite())
        assert "epc" in admin_obj.search_fields
        assert "tag_id" in admin_obj.search_fields


class TestRfidCoverReadAdminPermissions(TestCase):
    def test_no_add_permission(self):
        admin_obj = RfidCoverReadAdmin(RfidCoverTag, AdminSite())
        req = RequestFactory().get("/")
        req.user = MagicMock()
        assert admin_obj.has_add_permission(req) is False

    def test_no_change_permission(self):
        admin_obj = RfidCoverReadAdmin(RfidCoverTag, AdminSite())
        req = RequestFactory().get("/")
        req.user = MagicMock()
        assert admin_obj.has_change_permission(req) is False

    def test_no_delete_permission(self):
        admin_obj = RfidCoverReadAdmin(RfidCoverTag, AdminSite())
        req = RequestFactory().get("/")
        req.user = MagicMock()
        assert admin_obj.has_delete_permission(req) is False


class TestCoverStateTransitionAdminReadOnly(TestCase):
    def test_no_add_permission(self):
        admin_obj = CoverStateTransitionAdmin(RfidCoverTag, AdminSite())
        req = RequestFactory().get("/")
        req.user = MagicMock()
        assert admin_obj.has_add_permission(req) is False

    def test_no_change_permission(self):
        admin_obj = CoverStateTransitionAdmin(RfidCoverTag, AdminSite())
        req = RequestFactory().get("/")
        assert admin_obj.has_change_permission(req) is False

    def test_hash_preview(self):
        admin_obj = CoverStateTransitionAdmin(RfidCoverTag, AdminSite())
        fake = _Ns(prev_hash="abcdef123456abcdef", this_hash="999888777666555444")
        assert admin_obj.prev_hash_preview(fake).startswith("abcdef123456")
        assert admin_obj.this_hash_preview(fake).startswith("999888777666")


class TestCoverTamperAlertAdminActions(TestCase):
    def test_is_resolved_false(self):
        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        fake = _Ns(resolved_at=None)
        assert admin_obj.is_resolved(fake) is False

    def test_is_resolved_true(self):
        import datetime

        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        fake = _Ns(resolved_at=datetime.datetime(2026, 6, 1, tzinfo=tz.utc))
        assert admin_obj.is_resolved(fake) is True

    def test_list_filter_includes_severity(self):
        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        assert "severity" in admin_obj.list_filter

    def test_list_filter_includes_ecc_dispatched(self):
        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        assert "ecc_dispatched" in admin_obj.list_filter

    def test_list_filter_includes_resolved_status_filter(self):
        from inventory_admin.apps.rfid_covers.admin import ResolvedStatusFilter

        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        assert ResolvedStatusFilter in admin_obj.list_filter

    def test_bulk_resolve_action_registered(self):
        admin_obj = CoverTamperAlertAdmin(CoverTamperAlert, AdminSite())
        assert "bulk_resolve_with_note" in admin_obj.actions

    def test_mark_retired_action_registered(self):
        admin_obj = RfidCoverTagAdmin(RfidCoverTag, AdminSite())
        assert "mark_retired" in admin_obj.actions
