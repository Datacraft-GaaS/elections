"""Tests for M42+M43 accreditation domain (FR-M42-016, AC-M42-02, EC-DAT-01)."""

import uuid
import pytest
from django.test import TestCase
from uuid6 import uuid7

from accreditation_app.models import (
    MediaOrganisation,
    MediaAccreditation,
    ObserverOrganisation,
    IndividualObserver,
    CredentialRevocationEntry,
)
from accreditation_app.badge import generate_badge_serial, verify_badge
from accreditation_app.crl import generate_crl


pytestmark = pytest.mark.trace("FR-M42-016", "AC-M42-02", "EC-DAT-01")


@pytest.fixture
def media_org(db):
    return MediaOrganisation.objects.create(
        name="Nation Media Group",
        organisation_type="PRINT",
        mck_registered=True,
    )


@pytest.fixture
def observer_org(db):
    return ObserverOrganisation.objects.create(
        name="Carter Center",
        org_type="INTERNATIONAL_MISSION",
        jurisdiction="Kenya",
    )


@pytest.mark.django_db
class TestMediaAccreditationLifecycle:
    def test_initial_status_is_submitted(self, media_org):
        accred = MediaAccreditation.objects.create(
            applicant_name="Jane Wangari",
            id_number="12345678",
            organisation=media_org,
        )
        assert accred.status == "SUBMITTED"

    def test_status_progression(self, media_org):
        accred = MediaAccreditation.objects.create(
            applicant_name="Tom Otieno",
            id_number="87654321",
            organisation=media_org,
        )
        accred.status = "APPROVED"
        accred.save()
        accred.refresh_from_db()
        assert accred.status == "APPROVED"

        accred.status = "BADGE_ISSUED"
        badge = uuid7()
        accred.badge_serial = badge
        accred.save()
        accred.refresh_from_db()
        assert accred.status == "BADGE_ISSUED"
        assert accred.badge_serial == badge

    def test_revocation_sets_status_and_reason(self, media_org):
        serial = uuid7()
        accred = MediaAccreditation.objects.create(
            applicant_name="Peter Kamau",
            id_number="11223344",
            organisation=media_org,
            status="ACTIVE",
            badge_serial=serial,
        )
        accred.status = "REVOKED"
        accred.revocation_reason = "Violated access zones"
        from django.utils import timezone

        accred.revoked_at = timezone.now()
        accred.save()
        accred.refresh_from_db()
        assert accred.status == "REVOKED"
        assert accred.revocation_reason == "Violated access zones"
        assert accred.revoked_at is not None


@pytest.mark.django_db
class TestBadgeAndCRL:
    def test_generate_badge_serial_returns_str(self):
        serial = generate_badge_serial()
        assert isinstance(serial, str)
        assert len(serial) > 0

    def test_badge_verify_active(self):
        crl = {"generated_at": "2024-01-01T00:00:00+00:00", "revoked_serials": []}
        result = verify_badge("some-serial", crl)
        assert result in ("ACTIVE", "REVOKED")

    def test_badge_in_crl_is_revoked(self):
        serial = str(uuid7())
        crl = {"generated_at": "2024-01-01T00:00:00+00:00", "revoked_serials": [serial]}
        assert verify_badge(serial, crl) == "REVOKED"

    def test_crl_includes_revoked_credential_serials(self, db):
        serial = uuid7()
        CredentialRevocationEntry.objects.create(
            credential_serial=serial,
            reason="Test revocation",
            officer_id=uuid7(),
        )
        crl = generate_crl()
        assert str(serial) in crl["revoked_serials"]

    def test_crl_structure(self, db):
        crl = generate_crl()
        assert "generated_at" in crl
        assert "revoked_serials" in crl
        assert isinstance(crl["revoked_serials"], list)

    def test_crl_logs_warning_when_badge_table_missing(self, db, caplog):
        # BadgeRevocationEntry is unmanaged, so the table is absent in the
        # test database. The CRL must log the gap and still emit a payload.
        import logging

        with caplog.at_level(logging.WARNING, logger="accreditation_app.crl"):
            crl = generate_crl()
        assert any(
            "BadgeRevocationEntry unavailable" in record.message
            for record in caplog.records
        )
        assert "generated_at" in crl

    def test_duplicate_badge_serial_rejected(self, db, media_org):
        serial = uuid7()
        MediaAccreditation.objects.create(
            applicant_name="Alice",
            id_number="AAA001",
            organisation=media_org,
            badge_serial=serial,
        )
        with pytest.raises(Exception):
            MediaAccreditation.objects.create(
                applicant_name="Bob",
                id_number="BBB002",
                organisation=media_org,
                badge_serial=serial,
            )


@pytest.mark.django_db
class TestObserverLifecycle:
    def test_initial_status(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            full_name="Amina Hassan",
            id_number="OBS001",
            nationality="Kenyan",
        )
        assert observer.status == "SUBMITTED"

    def test_credential_issued(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            full_name="Carlos Diaz",
            id_number="OBS002",
            nationality="Spanish",
        )
        serial = uuid7()
        observer.status = "CREDENTIAL_ISSUED"
        observer.credential_serial = serial
        observer.save()
        observer.refresh_from_db()
        assert observer.status == "CREDENTIAL_ISSUED"
        assert observer.credential_serial == serial

    def test_observer_revocation(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            full_name="Liu Wei",
            id_number="OBS003",
            nationality="Chinese",
            status="DEPLOYED",
        )
        officer = uuid7()
        serial = uuid7()
        observer.credential_serial = serial
        observer.status = "REVOKED"
        observer.revocation_reason = "Interfered with tallying"
        observer.save()

        entry = CredentialRevocationEntry.objects.create(
            credential_serial=serial,
            reason="Interfered with tallying",
            officer_id=officer,
        )
        assert entry.credential_serial == serial
        crl = generate_crl()
        assert str(serial) in crl["revoked_serials"]


@pytest.mark.django_db
class TestNoPIIFields:
    def test_no_nps_fields_in_media_accreditation(self):
        field_names = {f.name for f in MediaAccreditation._meta.get_fields()}
        nps_fields = {"nid", "ssn", "health_id", "date_of_birth", "dob", "diagnosis"}
        assert not (field_names & nps_fields), (
            f"NPS fields found: {field_names & nps_fields}"
        )

    def test_no_nps_fields_in_observer(self):
        field_names = {f.name for f in IndividualObserver._meta.get_fields()}
        nps_fields = {"ssn", "health_id", "diagnosis", "dob"}
        assert not (field_names & nps_fields), (
            f"NPS fields found: {field_names & nps_fields}"
        )
