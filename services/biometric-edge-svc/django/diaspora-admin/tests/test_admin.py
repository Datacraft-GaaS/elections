"""Admin tests for diaspora admin (WP-5.8, M24)."""

from __future__ import annotations

import io
from datetime import datetime, timezone

import pytest
from django.contrib.auth.models import User
from django.test import Client

from diaspora_admin.apps.embassies.models import EmbassyStation, Form34ADiaspora
from diaspora_admin.apps.embassies.views import ImportReport, _validate_row, RowReport


@pytest.fixture
def staff_client(db) -> Client:
    user = User.objects.create_superuser("admin", "admin@test.local", "password")
    client = Client()
    client.force_login(user)
    return client


@pytest.fixture
def station(db) -> EmbassyStation:
    return EmbassyStation.objects.create(
        country_iso="GB",
        city="London",
        timezone="Europe/London",
        address="1 Kings Road",
        station_code="UK-LON-001",
        kits_deployed=1,
    )


# ---- Import view tests -------------------------------------------------------


@pytest.mark.django_db
def test_import_view_get_returns_form(staff_client: Client) -> None:
    response = staff_client.get("/embassies/import/")
    assert response.status_code == 200
    assert b"form" in response.content.lower()


@pytest.mark.django_db
def test_import_view_post_valid_csv(staff_client: Client) -> None:
    csv_content = (
        "id,national_id_last4,name_nfc,dob_yyyy,station_code\r\n"
        "V001,1234,Jane Doe,1980,UK-LON-001\r\n"
        "V002,5678,John Smith,1975,UK-LON-001\r\n"
    )
    f = io.BytesIO(csv_content.encode())
    f.name = "roll.csv"
    response = staff_client.post("/embassies/import/", {"csv_file": f})
    assert response.status_code == 200
    data = response.json()
    assert data["accepted"] == 2
    assert data["rejected"] == 0


@pytest.mark.django_db
def test_import_view_post_invalid_csv(staff_client: Client) -> None:
    csv_content = (
        "id,national_id_last4,name_nfc,dob_yyyy,station_code\r\n"
        "BAD,AB12,Jane Doe,abcd,\r\n"
    )
    f = io.BytesIO(csv_content.encode())
    f.name = "roll.csv"
    response = staff_client.post("/embassies/import/", {"csv_file": f})
    assert response.status_code == 200
    data = response.json()
    assert data["rejected"] == 1
    assert data["rows"][0]["ok"] is False


# ---- Admin changelist tests --------------------------------------------------


@pytest.mark.django_db
def test_embassy_station_admin_list(
    staff_client: Client, station: EmbassyStation
) -> None:
    response = staff_client.get("/admin/embassies/embassystation/")
    assert response.status_code == 200
    assert station.station_code.encode() in response.content


@pytest.mark.django_db
def test_form34a_admin_approve_action(
    staff_client: Client, station: EmbassyStation
) -> None:
    form = Form34ADiaspora.objects.create(
        embassy_station=station,
        form_id="FORM-ADMIN-001",
        results={"A": 10},
        rejected=0,
        disputed=0,
    )
    response = staff_client.post(
        "/admin/embassies/form34adiaspora/",
        {
            "action": "approve_forms",
            "_selected_action": [str(form.pk)],
        },
    )
    assert response.status_code in (200, 302)
    form.refresh_from_db()
    assert form.status == Form34ADiaspora.STATUS_APPROVED


# ---- Validate row unit tests -------------------------------------------------


def test_validate_row_valid() -> None:
    row = {
        "id": "V1",
        "national_id_last4": "1234",
        "name_nfc": "Jane",
        "dob_yyyy": "1980",
        "station_code": "US-DC-001",
    }
    result = _validate_row(2, row)
    assert result.ok


def test_validate_row_bad_national_id() -> None:
    row = {
        "id": "V1",
        "national_id_last4": "AB12",
        "name_nfc": "Jane",
        "dob_yyyy": "1980",
        "station_code": "US-DC-001",
    }
    result = _validate_row(2, row)
    assert not result.ok
    assert "national_id_last4" in result.error


def test_validate_row_bad_dob() -> None:
    row = {
        "id": "V1",
        "national_id_last4": "1234",
        "name_nfc": "Jane",
        "dob_yyyy": "1850",
        "station_code": "US-DC-001",
    }
    result = _validate_row(2, row)
    assert not result.ok
