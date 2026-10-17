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

"""Model unit tests for abis-reports-console."""

import pytest

from abis_reports_console.apps.reports.models import (
    ReportArtifact,
    ReportDefinition,
    ReportRun,
)


@pytest.mark.django_db
def test_definition_str(definition):
    assert str(definition) == definition.slug


@pytest.mark.django_db
def test_run_str(run, definition):
    assert definition.slug in str(run)


@pytest.mark.django_db
def test_artifact_sha256_hex(artifact):
    assert len(artifact.sha256_hex) == 64
    assert artifact.sha256_hex == "00" * 32


@pytest.mark.django_db
def test_artifact_size_human_kb(artifact):
    artifact.size_bytes = 2048
    assert "KB" in artifact.size_human or "B" in artifact.size_human


@pytest.mark.django_db
def test_artifact_size_human_mb():
    a = ReportArtifact.__new__(ReportArtifact)
    a.size_bytes = 3 * 1024 * 1024
    assert "MB" in a.size_human


@pytest.mark.django_db
def test_definition_active_filter(definition):
    """Retired definitions are excluded from the active filter."""
    active = ReportDefinition.objects.filter(retired_at__isnull=True)
    assert definition in active


@pytest.mark.django_db
def test_run_default_ordering(run):
    runs = ReportRun.objects.all()
    assert runs.first().pk == run.pk


@pytest.mark.django_db
def test_subscription_str(subscription):
    s = str(subscription)
    assert "Test Subscriber" in s or "test-daily-enrolment" in s
