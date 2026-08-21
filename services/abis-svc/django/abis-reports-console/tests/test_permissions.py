"""Permission tests for abis-reports-console."""

import pytest
from django.contrib.auth import get_user_model

User = get_user_model()


@pytest.mark.django_db
def test_spicedb_middleware_permits_staff(client):
    """Staff users pass the SpiceDB stub check."""
    user = User.objects.create_user("staff1", password="pass", is_staff=True)
    client.force_login(user)
    resp = client.get("/reports/runs/new/")
    assert resp.status_code == 200


@pytest.mark.django_db
def test_anonymous_denied_on_protected_view(client):
    resp = client.get("/reports/runs/new/")
    assert resp.status_code in (302, 401, 403)


@pytest.mark.django_db
def test_anonymous_denied_on_delivery_dashboard(client):
    resp = client.get("/reports/deliveries/")
    assert resp.status_code in (302, 401, 403)


@pytest.mark.django_db
def test_anonymous_denied_on_subscription_wizard(client):
    resp = client.get("/reports/subscriptions/new/")
    assert resp.status_code in (302, 401, 403)


@pytest.mark.django_db
def test_staff_can_access_admin(client):
    admin = User.objects.create_superuser("su", "su@test.local", "pass")
    client.force_login(admin)
    resp = client.get("/admin/eel_reports/reportdefinition/")
    assert resp.status_code == 200


@pytest.mark.django_db
def test_non_staff_cannot_access_admin(client):
    user = User.objects.create_user("nobody", password="pass", is_staff=False)
    client.force_login(user)
    resp = client.get("/admin/")
    # Django admin redirects non-staff to login.
    assert resp.status_code in (302, 403)
