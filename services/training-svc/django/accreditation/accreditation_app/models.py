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

"""Models for M42 (media accreditation) and M43 (observer accreditation).

Every `db_table` here is created by services/training-svc/migrations/
0003_training_accreditation.sql, which the deploy applies. The models describe
that schema; they do not define it. They previously described a different one
-- `applicant_name`, `photo_url`, `revocation_reason` and eleven more columns
existed only here -- and nothing compared the two, so the ORM would have raised
on the first query against a real database.

Two absences are deliberate, not oversights. `full_name` and `id_number` are
data class B, held in a separate encrypted vault; the SQL keeps them out of
`training_accred_observers` on purpose, so they are not modelled here.
`badge_serial` and `credential_serial` are TEXT in the SQL, not UUID.
"""

from django.contrib.postgres.fields import ArrayField
from django.db import models
from uuid6 import uuid7


def uuid7str() -> str:
    return str(uuid7())


class MediaOrganisation(models.Model):
    ORG_TYPE_CHOICES = [
        ("PRINT", "Print"),
        ("BROADCAST", "Broadcast"),
        ("DIGITAL", "Digital"),
        ("AGENCY", "Agency"),
    ]
    STATUS_CHOICES = [
        ("pending", "Pending"),
        ("approved", "Approved"),
        ("suspended", "Suspended"),
        ("revoked", "Revoked"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    name = models.CharField(max_length=500)
    mck_registered = models.BooleanField(default=False)
    status = models.CharField(max_length=30, choices=STATUS_CHOICES, default="pending")
    mou_signed_at = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "training_accred_media_orgs"
        managed = True

    def __str__(self) -> str:
        return self.name


class MediaAccreditation(models.Model):
    STATUS_CHOICES = [
        ("pending", "Pending"),
        ("approved", "Approved"),
        ("badge_issued", "Badge Issued"),
        ("revoked", "Revoked"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    organisation = models.ForeignKey(
        MediaOrganisation,
        on_delete=models.CASCADE,
        related_name="accreditations",
        db_column="organisation_id",
    )
    mck_membership_number = models.CharField(max_length=50)
    # TEXT[] in the SQL, so ArrayField. A JSONField here raises against the real
    # table ("column access_zones is of type text[] but expression is of type
    # jsonb"), and ArrayField is Postgres-only -- which is why this app's suite
    # runs on Postgres rather than SQLite.
    access_zones = ArrayField(models.TextField(), default=list, blank=True)
    badge_serial = models.CharField(max_length=64, null=True, blank=True, unique=True)
    badge_issued_at = models.DateTimeField(null=True, blank=True)
    badge_hsm_signature = models.TextField(null=True, blank=True)
    status = models.CharField(max_length=30, choices=STATUS_CHOICES, default="pending")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "training_accred_media_accreditations"
        managed = True

    def __str__(self) -> str:
        return f"{self.mck_membership_number} ({self.status})"


class BadgeRevocationEntry(models.Model):
    """One revoked media badge.

    `generate_crl()` in crl.py reads this table to build the certificate
    revocation list, so a revocation that is not written here leaves the badge
    reported valid. The view writes it; nothing else does.
    """

    REASON_CHOICES = [
        ("BREACH", "Breach"),
        ("MISCONDUCT", "Misconduct"),
        ("SECURITY", "Security"),
        ("EXPIRED", "Expired"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    accreditation = models.ForeignKey(
        MediaAccreditation,
        on_delete=models.CASCADE,
        related_name="revocations",
        db_column="accreditation_id",
    )
    badge_serial = models.CharField(max_length=64)
    officer_id = models.UUIDField()
    reason = models.CharField(max_length=30, choices=REASON_CHOICES)
    revoked_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "training_accred_badge_revocations"
        managed = True

    def __str__(self) -> str:
        return f"BadgeRevocation({self.badge_serial})"


class AccreditationAuditEntry(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    actor_id = models.UUIDField()
    entity_type = models.CharField(max_length=50)
    entity_id = models.UUIDField()
    action = models.CharField(max_length=50)
    metadata = models.JSONField(default=dict)
    recorded_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "training_accred_audit_entries"
        managed = True

    def __str__(self) -> str:
        return f"{self.action} {self.entity_type}/{self.entity_id}"


class ObserverOrganisation(models.Model):
    ORG_TYPE_CHOICES = [
        ("DOMESTIC_NGO", "Domestic NGO"),
        ("INTERNATIONAL_MISSION", "International Mission"),
        ("DIPLOMATIC_MISSION", "Diplomatic Mission"),
        ("RELIGIOUS_BODY", "Religious Body"),
        ("ACADEMIC", "Academic"),
    ]
    STATUS_CHOICES = [
        ("pending", "Pending"),
        ("approved", "Approved"),
        ("suspended", "Suspended"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    name = models.CharField(max_length=500)
    nationality = models.CharField(max_length=100)
    org_type = models.CharField(max_length=30, choices=ORG_TYPE_CHOICES)
    mou_signed_at = models.DateTimeField(null=True, blank=True)
    status = models.CharField(max_length=30, choices=STATUS_CHOICES, default="pending")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "training_accred_observer_orgs"
        managed = True

    def __str__(self) -> str:
        return self.name


class IndividualObserver(models.Model):
    STATUS_CHOICES = [
        ("pending", "Pending"),
        ("approved", "Approved"),
        ("deployed", "Deployed"),
        ("credential_revoked", "Credential Revoked"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    organisation = models.ForeignKey(
        ObserverOrganisation,
        on_delete=models.CASCADE,
        related_name="observers",
        db_column="organisation_id",
    )
    nationality = models.CharField(max_length=100)
    deployment_county_codes = ArrayField(models.TextField(), default=list, blank=True)
    credential_serial = models.CharField(max_length=64, null=True, blank=True, unique=True)
    credential_issued_at = models.DateTimeField(null=True, blank=True)
    credential_hsm_signature = models.TextField(null=True, blank=True)
    status = models.CharField(max_length=30, choices=STATUS_CHOICES, default="pending")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "training_accred_observers"
        managed = True

    def __str__(self) -> str:
        return f"Observer({self.credential_serial}) ({self.status})"


class CredentialRevocationEntry(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    observer = models.ForeignKey(
        IndividualObserver,
        on_delete=models.CASCADE,
        related_name="credential_revocations",
        db_column="observer_id",
    )
    credential_serial = models.CharField(max_length=64)
    officer_id = models.UUIDField()
    reason = models.TextField()
    revoked_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "training_accred_credential_revocations"
        managed = True

    def __str__(self) -> str:
        return f"CredentialRevocation({self.credential_serial})"
