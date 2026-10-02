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

"""View tests for abis-reports-console."""

import uuid
from datetime import datetime, timezone

import pytest
from django.contrib.auth import get_user_model
from django.test import Client

User = get_user_model()


def _make_staff(username="staff"):
    user = User.objects.create_user(username=username, password="pass", is_staff=True)
    return user


@pytest.mark.django_db
def test_new_run_get_renders(client):
    user = _make_staff()
    client.force_login(user)
    resp = client.get("/reports/runs/new/")
    assert resp.status_code == 200
    assert b"Queue a New Report Run" in resp.content


@pytest.mark.django_db
def test_new_run_post_missing_slug(client):
    user = _make_staff()
    client.force_login(user)
    resp = client.post(
        "/reports/runs/new/",
        {
            "window_from": "2025-01-01T00:00",
            "window_to": "2025-01-02T00:00",
            "formats": ["pdf"],
        },
    )
    assert resp.status_code == 400
    assert b"Slug is required" in resp.content


@pytest.mark.django_db
def test_new_run_post_creates_queued_run(client, definition):
    user = _make_staff()
    client.force_login(user)
    resp = client.post(
        "/reports/runs/new/",
        {
            "slug": definition.slug,
            "window_from": "2025-01-01T00:00",
            "window_to": "2025-01-02T00:00",
            "formats": ["pdf", "csv"],
        },
    )
    # Redirects to run detail on success.
    assert resp.status_code in (200, 302)


@pytest.mark.django_db
def test_run_detail_htmx_partial(client, run):
    user = _make_staff()
    client.force_login(user)
    resp = client.get(f"/reports/runs/{run.run_id}/", HTTP_HX_REQUEST="true")
    assert resp.status_code == 200
    assert run.slug.encode() in resp.content


@pytest.mark.django_db
def test_run_detail_invalid_id_returns_404(client):
    user = _make_staff()
    client.force_login(user)
    resp = client.get(f"/reports/runs/{uuid.uuid4()}/")
    assert resp.status_code == 404


@pytest.mark.django_db
def test_delivery_dashboard_renders(client):
    user = _make_staff()
    client.force_login(user)
    resp = client.get("/reports/deliveries/")
    assert resp.status_code == 200
    assert b"Delivery Dashboard" in resp.content


@pytest.mark.django_db
def test_subscription_wizard_get(client):
    user = _make_staff()
    client.force_login(user)
    resp = client.get("/reports/subscriptions/new/")
    assert resp.status_code == 200
    assert b"Subscription Wizard" in resp.content


@pytest.mark.django_db
def test_subscription_wizard_post_missing_subscriber(client, definition):
    user = _make_staff()
    client.force_login(user)
    resp = client.post(
        "/reports/subscriptions/new/",
        {
            "definition_id": str(definition.definition_id),
            "format": "pdf",
        },
    )
    assert resp.status_code == 400
    assert b"Subscriber is required" in resp.content


@pytest.mark.django_db
def test_unauthenticated_redirects_to_login(client):
    resp = client.get("/reports/runs/new/")
    assert resp.status_code in (302, 301)
    assert "/login" in resp["Location"] or "login" in resp["Location"]
