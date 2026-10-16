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

"""Tests for the admin configuration."""

import uuid

import pytest
from django.contrib.admin.sites import AdminSite
from django.utils import timezone

from abis_console.apps.adjudication.admin import AdjudicationCaseAdmin
from abis_console.apps.adjudication.models import AdjudicationCase


def make_case_obj(state="Open", priority="Standard"):
    from datetime import timedelta

    case = AdjudicationCase.__new__(AdjudicationCase)
    case.id = uuid.uuid4()
    case.opened_at = timezone.now()
    case.priority = priority
    case.tier = "Tier1Analyst"
    case.state = state
    case.enrolment_job_id = uuid.uuid4()
    case.probable_match_subject_ids = []
    case.evidence_bundle_ref = uuid.uuid4()
    case.assigned_to = None
    case.assigned_at = None
    case.decided_at = None
    case.decision_type = None
    case.decision_subject_id = None
    case.decision_reason = None
    case.sla_deadline_at = timezone.now() + timedelta(hours=4)
    case.created_at = timezone.now()
    case.updated_at = timezone.now()
    return case


@pytest.mark.django_db
class TestAdjudicationCaseAdmin:
    def test_list_display_includes_priority(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "priority" in admin.list_display

    def test_list_display_includes_state(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "state" in admin.list_display

    def test_list_display_includes_sla_hours_remaining(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "sla_hours_remaining_display" in admin.list_display

    def test_list_filter_includes_state(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "state" in admin.list_filter

    def test_list_filter_includes_priority(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "priority" in admin.list_filter

    def test_list_filter_includes_tier(self):
        admin = AdjudicationCaseAdmin(AdjudicationCase, AdminSite())
        assert "tier" in admin.list_filter

    def test_decided_case_has_extra_readonly_fields(self):
        site = AdminSite()
        admin_obj = AdjudicationCaseAdmin(AdjudicationCase, site)
        request = type("R", (), {"user": None})()
        case = make_case_obj(state="Decided")
        ro = admin_obj.get_readonly_fields(request, obj=case)
        assert "decision_type" in ro
        assert "state" in ro

    def test_open_case_does_not_lock_decision_fields(self):
        site = AdminSite()
        admin_obj = AdjudicationCaseAdmin(AdjudicationCase, site)
        request = type("R", (), {"user": None})()
        case = make_case_obj(state="Open")
        ro = admin_obj.get_readonly_fields(request, obj=case)
        assert "decision_type" not in ro
