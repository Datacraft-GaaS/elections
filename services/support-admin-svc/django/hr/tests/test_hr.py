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

"""Tests for M55 HR-Payroll Django app (WP-5.6)."""

import uuid
from datetime import datetime, timezone

import pytest
from django.test import TestCase
from django.urls import reverse
from rest_framework.test import APIClient

from hr_app.models import (
    ApplicationStatus,
    JobApplication,
    PayrollRun,
    PayrollStatus,
)


def _uuid() -> str:
    """Generate a UUID7 string (falls back to UUID4 if uuid6 unavailable)."""
    try:
        from uuid6 import uuid7

        return str(uuid7())
    except ImportError:
        return str(uuid.uuid4())


def _now() -> datetime:
    return datetime.now(tz=timezone.utc)


class JobApplicationModelTest(TestCase):
    def test_create_application(self) -> None:
        app = JobApplication.objects.create(
            id=_uuid(),
            role_id=_uuid(),
            applicant_name="Alice Wanjiku",
            national_id_hash="aabbcc",
            status=ApplicationStatus.APPLIED,
            applied_at=_now(),
            training_gate_passed=False,
        )
        assert app.pk is not None
        assert app.status == ApplicationStatus.APPLIED
        assert app.training_gate_passed is False

    def test_shortlisted_status(self) -> None:
        app = JobApplication.objects.create(
            id=_uuid(),
            role_id=_uuid(),
            applicant_name="Bob Otieno",
            national_id_hash="ddeeff",
            status=ApplicationStatus.SHORTLISTED,
            applied_at=_now(),
            training_gate_passed=True,
        )
        assert app.status == ApplicationStatus.SHORTLISTED


class PayrollRunModelTest(TestCase):
    def test_create_payroll_run(self) -> None:
        run = PayrollRun.objects.create(
            id=_uuid(),
            election_id=_uuid(),
            period="2027-08",
            status=PayrollStatus.DRAFT,
            total_staff=0,
            total_gross_kes=0,
            initiated_by=_uuid(),
            started_at=_now(),
        )
        assert run.pk is not None
        assert run.status == PayrollStatus.DRAFT
        assert run.approved_by is None


class ApplicationAPITest(TestCase):
    def setUp(self) -> None:
        self.client = APIClient()

    def test_list_applications_empty(self) -> None:
        response = self.client.get("/v1/hr/applications/")
        assert response.status_code == 200

    def test_create_application(self) -> None:
        payload = {
            "id": _uuid(),
            "role_id": _uuid(),
            "applicant_name": "Carol Muthoni",
            "national_id_hash": "112233",
            "status": "applied",
            "applied_at": _now().isoformat(),
            "training_gate_passed": False,
        }
        response = self.client.post("/v1/hr/applications/", payload, format="json")
        assert response.status_code == 201
        assert response.data["applicant_name"] == "Carol Muthoni"


class PayrollRunAPITest(TestCase):
    def setUp(self) -> None:
        self.client = APIClient()

    def test_list_runs_empty(self) -> None:
        response = self.client.get("/v1/payroll/runs/")
        assert response.status_code == 200

    def test_create_run(self) -> None:
        payload = {
            "id": _uuid(),
            "election_id": _uuid(),
            "period": "2027-08",
            "status": "draft",
            "total_staff": 0,
            "total_gross_kes": 0,
            "initiated_by": _uuid(),
            "started_at": _now().isoformat(),
        }
        response = self.client.post("/v1/payroll/runs/", payload, format="json")
        assert response.status_code == 201
        assert response.data["period"] == "2027-08"
