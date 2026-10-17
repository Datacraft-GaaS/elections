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

"""Tests for adjudication views."""

import hashlib
import io
import uuid
from datetime import timedelta
from unittest.mock import patch

import pytest
from django.contrib.auth.models import User
from django.test import Client
from django.urls import reverse
from django.utils import timezone


def make_case_mock(state="Open", tier="Tier1Analyst", priority="Standard"):
    from abis_console.apps.adjudication.models import AdjudicationCase

    case = AdjudicationCase.__new__(AdjudicationCase)
    case.pk = uuid.uuid4()
    case.id = case.pk
    case.opened_at = timezone.now()
    case.priority = priority
    case.tier = tier
    case.state = state
    case.enrolment_job_id = uuid.uuid4()
    case.probable_match_subject_ids = [str(uuid.uuid4())]
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


@pytest.fixture
def auth_client(db):
    User.objects.create_user(username="reviewer1", password="testpass")
    client = Client()
    client.login(username="reviewer1", password="testpass")
    return client


@pytest.mark.django_db
class TestCaseListView:
    def test_redirects_unauthenticated(self, client):
        url = reverse("adjudication:case_list")
        response = client.get(url)
        assert response.status_code == 302
        assert "/login" in response["Location"] or "login" in response["Location"]

    def test_case_list_renders_for_authenticated_user(self, auth_client):
        url = reverse("adjudication:case_list")
        with patch(
            "abis_console.apps.adjudication.views.AdjudicationCase.objects"
        ) as mock_qs:
            mock_qs.exclude.return_value.order_by.return_value = []
            response = auth_client.get(url)
        assert response.status_code == 200

    def test_case_list_uses_correct_template(self, auth_client):
        url = reverse("adjudication:case_list")
        with patch(
            "abis_console.apps.adjudication.views.AdjudicationCase.objects"
        ) as mock_qs:
            mock_qs.exclude.return_value.order_by.return_value = []
            response = auth_client.get(url)
        assert "adjudication/case_list.html" in [t.name for t in response.templates]


@pytest.mark.django_db
class TestCaseDetailView:
    def test_case_detail_renders(self, auth_client):
        case = make_case_mock()
        url = reverse("adjudication:case_detail", kwargs={"case_id": str(case.id)})
        with (
            patch(
                "abis_console.apps.adjudication.views.get_object_or_404",
                return_value=case,
            ),
            patch(
                "abis_console.apps.adjudication.views.ReviewerAction.objects"
            ) as mock_ra,
        ):
            mock_ra.filter.return_value.order_by.return_value = []
            response = auth_client.get(url)
        assert response.status_code == 200

    def test_htmx_case_detail_returns_partial(self, auth_client):
        case = make_case_mock()
        url = reverse("adjudication:case_detail", kwargs={"case_id": str(case.id)})
        with (
            patch(
                "abis_console.apps.adjudication.views.get_object_or_404",
                return_value=case,
            ),
            patch(
                "abis_console.apps.adjudication.views.ReviewerAction.objects"
            ) as mock_ra,
        ):
            mock_ra.filter.return_value.order_by.return_value = []
            response = auth_client.get(url, HTTP_HX_REQUEST="true")
        assert response.status_code == 200
        assert "adjudication/partials/case_detail_panel.html" in [
            t.name for t in response.templates
        ]


@pytest.mark.django_db
class TestDecisionView:
    def test_decision_form_requires_decision_type(self, auth_client):
        case = make_case_mock()
        url = reverse("adjudication:submit_decision", kwargs={"case_id": str(case.id)})
        with patch(
            "abis_console.apps.adjudication.views.get_object_or_404", return_value=case
        ):
            response = auth_client.post(url, {})
        assert response.status_code == 400

    def test_tier3_decision_requires_second_reviewer(self, auth_client):
        case = make_case_mock(tier="Tier3Panel")
        url = reverse("adjudication:submit_decision", kwargs={"case_id": str(case.id)})
        with patch(
            "abis_console.apps.adjudication.views.get_object_or_404", return_value=case
        ):
            response = auth_client.post(url, {"decision_type": "ConfirmNoMatch"})
        assert response.status_code == 400

    def test_decided_case_returns_400(self, auth_client):
        case = make_case_mock(state="Decided")
        url = reverse("adjudication:submit_decision", kwargs={"case_id": str(case.id)})
        with patch(
            "abis_console.apps.adjudication.views.get_object_or_404", return_value=case
        ):
            response = auth_client.post(url, {"decision_type": "ConfirmNoMatch"})
        assert response.status_code == 400


@pytest.mark.django_db
class TestEvidenceUploadView:
    def test_evidence_upload_computes_hash(self, auth_client):
        case = make_case_mock()
        url = reverse("adjudication:upload_evidence", kwargs={"case_id": str(case.id)})
        content = b"fake biometric evidence bytes"
        expected_hash = hashlib.sha256(content).hexdigest()
        evidence_file = io.BytesIO(content)
        evidence_file.name = "evidence.png"

        with patch(
            "abis_console.apps.adjudication.views.get_object_or_404", return_value=case
        ):
            response = auth_client.post(
                url, {"evidence": evidence_file}, format="multipart"
            )
        assert response.status_code == 200
        assert expected_hash in response.content.decode()

    def test_evidence_upload_without_file_returns_400(self, auth_client):
        case = make_case_mock()
        url = reverse("adjudication:upload_evidence", kwargs={"case_id": str(case.id)})
        with patch(
            "abis_console.apps.adjudication.views.get_object_or_404", return_value=case
        ):
            response = auth_client.post(url, {})
        assert response.status_code == 400
