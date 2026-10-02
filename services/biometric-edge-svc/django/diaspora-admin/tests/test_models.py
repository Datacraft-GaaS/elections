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

"""Model tests for diaspora admin (WP-5.8, M24)."""

from __future__ import annotations

import uuid
from datetime import datetime, timezone

import pytest

from diaspora_admin.apps.embassies.models import (
    CustodyEvent,
    DiasporaRoll,
    EmbassyStation,
    Form34ADiaspora,
    PouchCustody,
)


@pytest.fixture
def station(db) -> EmbassyStation:
    return EmbassyStation.objects.create(
        country_iso="US",
        city="Washington",
        timezone="America/New_York",
        address="123 K Street NW",
        station_code="US-DC-001",
        kits_deployed=2,
    )


@pytest.mark.django_db
def test_embassy_station_created(station: EmbassyStation) -> None:
    assert station.pk is not None
    assert station.station_code == "US-DC-001"
    assert not station.is_open


@pytest.mark.django_db
def test_embassy_station_is_open_when_opened(station: EmbassyStation) -> None:
    now = datetime.now(tz=timezone.utc)
    station.opened_at = now
    station.save()
    station.refresh_from_db()
    assert station.is_open


@pytest.mark.django_db
def test_embassy_station_closed_when_closed_at_set(station: EmbassyStation) -> None:
    now = datetime.now(tz=timezone.utc)
    station.opened_at = now
    station.closed_at = now
    station.save()
    assert not station.is_open


@pytest.mark.django_db
def test_diaspora_roll_created(station: EmbassyStation) -> None:
    roll = DiasporaRoll.objects.create(
        embassy_station=station,
        snapshot_taken_at=datetime.now(tz=timezone.utc),
        voter_count=500,
        hash="a" * 64,
        frozen=False,
    )
    assert roll.pk is not None
    assert roll.voter_count == 500
    assert not roll.frozen


@pytest.mark.django_db
def test_form34a_default_status_pending(station: EmbassyStation) -> None:
    form = Form34ADiaspora.objects.create(
        embassy_station=station,
        form_id="FORM-001",
        results={"A": 100, "B": 80},
        rejected=5,
        disputed=2,
    )
    assert form.status == Form34ADiaspora.STATUS_PENDING
    assert form.transmitted_at is None


@pytest.mark.django_db
def test_pouch_custody_chain_ordering(station: EmbassyStation) -> None:
    pouch = PouchCustody.objects.create(
        form_ids=[str(uuid.uuid4())],
        seal_id="SEAL-001",
        sealed_at=datetime.now(tz=timezone.utc),
    )
    for i in range(3):
        CustodyEvent.objects.create(
            pouch=pouch,
            transferred_at=datetime.now(tz=timezone.utc),
            from_person=f"Officer-{i}",
            to_person=f"Officer-{i + 1}",
            location="Airport",
            gps_lat=-1.28,
            gps_lon=36.82,
            signature="sig",
            seq=i,
        )
    events = list(pouch.custody_events.all())
    assert len(events) == 3
    for i, event in enumerate(events):
        assert event.seq == i
        assert event.from_person == f"Officer-{i}"


@pytest.mark.django_db
def test_form34a_approve_updates_status(station: EmbassyStation) -> None:
    form = Form34ADiaspora.objects.create(
        embassy_station=station,
        form_id="FORM-002",
        results={"X": 50},
        rejected=0,
        disputed=0,
    )
    Form34ADiaspora.objects.filter(pk=form.pk).update(
        status=Form34ADiaspora.STATUS_APPROVED
    )
    form.refresh_from_db()
    assert form.status == Form34ADiaspora.STATUS_APPROVED
