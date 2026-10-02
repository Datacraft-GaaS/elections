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

"""Pytest fixtures for abis-reports-console tests.

Unmanaged models are patched to managed=True so the SQLite test runner creates
the tables. This patching must happen before Django creates the test database,
which means it runs at module import time (conftest.py is collected before DB setup).
"""

import os
import uuid
from datetime import datetime, timezone

# Ensure the test-sqlite env var is set before Django settings are loaded.
os.environ.setdefault("DJANGO_TEST_SQLITE", "true")

import django

django.setup()

from abis_reports_console.apps.reports.models import (
    ReportAccessLog,
    ReportArtifact,
    ReportDefinition,
    ReportDelivery,
    ReportRun,
    ReportSubscriber,
    ReportSubscription,
    ReportTlogAnchor,
)

# Patch managed=True so pytest-django creates tables in the test SQLite DB.
for _model in (
    ReportDefinition,
    ReportRun,
    ReportArtifact,
    ReportTlogAnchor,
    ReportSubscriber,
    ReportSubscription,
    ReportDelivery,
    ReportAccessLog,
):
    _model._meta.managed = True

import pytest


def _now():
    return datetime.now(tz=timezone.utc)


@pytest.fixture
def definition(db):
    return ReportDefinition.objects.create(
        definition_id=uuid.uuid4(),
        slug="test-daily-enrolment",
        title="Test Daily Enrolment",
        report_class="operational",
        audience=["commission_staff"],
        data_class="sensitive",
        renderer_pipeline=["json", "csv", "pdf"],
        retention_years=5,
        template_version="1.0.0",
        created_at=_now(),
        updated_at=_now(),
    )


@pytest.fixture
def run(db, definition):
    return ReportRun.objects.create(
        run_id=uuid.uuid4(),
        definition=definition,
        slug=definition.slug,
        window_from=_now(),
        window_to=_now(),
        status="complete",
        requested_by="testuser",
        requested_at=_now(),
        rows_extracted=1000,
        total_ms=420,
    )


@pytest.fixture
def artifact(db, run):
    return ReportArtifact.objects.create(
        artifact_id=uuid.uuid4(),
        run=run,
        format="pdf",
        blob_ref="reports/test/report.pdf",
        sha256=b"\x00" * 32,
        size_bytes=102400,
        page_count=4,
        rendered_at=_now(),
    )


@pytest.fixture
def subscriber(db):
    return ReportSubscriber.objects.create(
        subscriber_id=uuid.uuid4(),
        name="Test Subscriber",
        channel="email",
        endpoint="test@example.org",
        endpoint_hash=b"\x00" * 32,
        audience="commission_staff",
        preferred_format="pdf",
        created_at=_now(),
    )


@pytest.fixture
def subscription(db, subscriber, definition):
    return ReportSubscription.objects.create(
        subscription_id=uuid.uuid4(),
        subscriber=subscriber,
        definition=definition,
        format="pdf",
        filter={},
        created_at=_now(),
    )
