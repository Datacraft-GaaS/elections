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

"""Tests for M42+M43 accreditation domain (FR-M42-016, AC-M42-02, EC-DAT-01).

These exercise the models and the CRL. The suite runs on PostgreSQL, because
the app's array columns are ArrayField; it still builds its tables from the
models, so it cannot tell whether the models match
services/training-svc/migrations/0003_training_accreditation.sql.
`tools/check-django-schema-drift.py` is what compares those two.
"""

import pytest
from uuid6 import uuid7

from accreditation_app.badge import generate_badge_serial, verify_badge
from accreditation_app.crl import generate_crl
from accreditation_app.models import (
    BadgeRevocationEntry,
    CredentialRevocationEntry,
    IndividualObserver,
    MediaAccreditation,
    MediaOrganisation,
    ObserverOrganisation,
)

pytestmark = pytest.mark.trace("FR-M42-016", "AC-M42-02", "EC-DAT-01")


@pytest.fixture
def media_org(db):
    return MediaOrganisation.objects.create(
        name="Nation Media Group",
        mck_registered=True,
    )


@pytest.fixture
def observer_org(db):
    return ObserverOrganisation.objects.create(
        name="Carter Center",
        org_type="INTERNATIONAL_MISSION",
        nationality="Kenya",
    )


@pytest.mark.django_db
class TestMediaAccreditationLifecycle:
    def test_initial_status_is_pending(self, media_org):
        accred = MediaAccreditation.objects.create(
            mck_membership_number="MCK-0001",
            organisation=media_org,
        )
        assert accred.status == "pending"

    def test_status_progression(self, media_org):
        accred = MediaAccreditation.objects.create(
            mck_membership_number="MCK-0002",
            organisation=media_org,
        )
        accred.status = "approved"
        accred.save()
        accred.refresh_from_db()
        assert accred.status == "approved"

        accred.status = "badge_issued"
        accred.badge_serial = "BADGE-0002"
        accred.save()
        accred.refresh_from_db()
        assert accred.status == "badge_issued"
        assert accred.badge_serial == "BADGE-0002"

    def test_revocation_writes_a_crl_entry(self, media_org):
        """A revocation that stops at `status` does not revoke the badge.

        `generate_crl()` builds the revocation list from BadgeRevocationEntry,
        so the row is the revocation. This asserted only the status field
        before, which is why the missing insert went unnoticed.
        """
        accred = MediaAccreditation.objects.create(
            mck_membership_number="MCK-0003",
            organisation=media_org,
            status="badge_issued",
            badge_serial="BADGE-0003",
        )
        officer = uuid7()
        BadgeRevocationEntry.objects.create(
            accreditation=accred,
            badge_serial=accred.badge_serial,
            officer_id=officer,
            reason="BREACH",
        )
        accred.status = "revoked"
        accred.save()

        crl = generate_crl()
        assert "BADGE-0003" in crl["revoked_serials"]
        assert verify_badge("BADGE-0003", crl) == "REVOKED"

    def test_duplicate_badge_serial_rejected(self, db, media_org):
        MediaAccreditation.objects.create(
            mck_membership_number="MCK-0004",
            organisation=media_org,
            badge_serial="BADGE-DUP",
        )
        with pytest.raises(Exception):
            MediaAccreditation.objects.create(
                mck_membership_number="MCK-0005",
                organisation=media_org,
                badge_serial="BADGE-DUP",
            )


@pytest.mark.django_db
class TestBadgeAndCRL:
    def test_generate_badge_serial_returns_str(self):
        serial = generate_badge_serial()
        assert isinstance(serial, str)
        assert len(serial) > 0

    def test_badge_verify_active(self, db, media_org):
        MediaAccreditation.objects.create(
            mck_membership_number="MCK-ACTIVE",
            organisation=media_org,
            status="badge_issued",
            badge_serial="BADGE-ACTIVE",
        )
        crl = {"generated_at": "2024-01-01T00:00:00+00:00", "revoked_serials": []}
        assert verify_badge("BADGE-ACTIVE", crl) == "ACTIVE"

    def test_unknown_badge_is_revoked_not_active(self, db):
        """An unknown serial must not verify as valid.

        The badge table is empty here, which is also what a lookup against a
        missing row returns -- the case the old `except: return "ACTIVE"`
        turned into a pass.
        """
        crl = {"generated_at": "2024-01-01T00:00:00+00:00", "revoked_serials": []}
        assert verify_badge("no-such-badge", crl) == "REVOKED"

    def test_badge_in_crl_is_revoked(self):
        crl = {"generated_at": "2024-01-01T00:00:00+00:00", "revoked_serials": ["s-1"]}
        assert verify_badge("s-1", crl) == "REVOKED"

    def test_crl_includes_revoked_credential_serials(self, db, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            nationality="Kenyan",
            credential_serial="CRED-0001",
        )
        CredentialRevocationEntry.objects.create(
            observer=observer,
            credential_serial="CRED-0001",
            reason="Test revocation",
            officer_id=uuid7(),
        )
        crl = generate_crl()
        assert "CRED-0001" in crl["revoked_serials"]

    def test_crl_includes_revoked_badge_serials(self, db, media_org):
        accred = MediaAccreditation.objects.create(
            mck_membership_number="MCK-0006",
            organisation=media_org,
            badge_serial="BADGE-0006",
        )
        BadgeRevocationEntry.objects.create(
            accreditation=accred,
            badge_serial="BADGE-0006",
            officer_id=uuid7(),
            reason="SECURITY",
        )
        crl = generate_crl()
        assert "BADGE-0006" in crl["revoked_serials"]

    def test_crl_structure(self, db):
        crl = generate_crl()
        assert "generated_at" in crl
        assert "revoked_serials" in crl
        assert isinstance(crl["revoked_serials"], list)


@pytest.mark.django_db
class TestObserverLifecycle:
    def test_initial_status(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            nationality="Kenyan",
        )
        assert observer.status == "pending"

    def test_credential_issued(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            nationality="Spanish",
        )
        observer.status = "approved"
        observer.credential_serial = "CRED-0002"
        observer.save()
        observer.refresh_from_db()
        assert observer.status == "approved"
        assert observer.credential_serial == "CRED-0002"

    def test_observer_revocation_reaches_the_crl(self, observer_org):
        observer = IndividualObserver.objects.create(
            organisation=observer_org,
            nationality="Chinese",
            status="deployed",
            credential_serial="CRED-0003",
        )
        officer = uuid7()
        observer.status = "credential_revoked"
        observer.save()

        CredentialRevocationEntry.objects.create(
            observer=observer,
            credential_serial="CRED-0003",
            reason="Interfered with tallying",
            officer_id=officer,
        )
        crl = generate_crl()
        assert "CRED-0003" in crl["revoked_serials"]
