import uuid
from uuid6 import uuid7
from django.db import models


def uuid7str() -> str:
    return str(uuid7())


class MediaOrganisation(models.Model):
    ORG_TYPE_CHOICES = [
        ("PRINT", "Print"),
        ("BROADCAST", "Broadcast"),
        ("DIGITAL", "Digital"),
        ("AGENCY", "Agency"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    name = models.CharField(max_length=500)
    organisation_type = models.CharField(max_length=30, choices=ORG_TYPE_CHOICES)
    mck_registered = models.BooleanField(default=False)
    live_feed_permit = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "training_accred_media_orgs"
        managed = True

    def __str__(self) -> str:
        return self.name


class MediaAccreditation(models.Model):
    STATUS_CHOICES = [
        ("SUBMITTED", "Submitted"),
        ("UNDER_VETTING", "Under Vetting"),
        ("APPROVED", "Approved"),
        ("REJECTED", "Rejected"),
        ("BADGE_ISSUED", "Badge Issued"),
        ("ACTIVE", "Active"),
        ("SUSPENDED", "Suspended"),
        ("REVOKED", "Revoked"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    applicant_name = models.CharField(max_length=500)
    id_number = models.CharField(max_length=50)
    organisation = models.ForeignKey(
        MediaOrganisation,
        on_delete=models.CASCADE,
        related_name="accreditations",
    )
    mck_membership_number = models.CharField(max_length=50, blank=True)
    photo_url = models.URLField(blank=True)
    access_zones = models.JSONField(default=list)
    status = models.CharField(
        max_length=30, choices=STATUS_CHOICES, default="SUBMITTED"
    )
    badge_serial = models.UUIDField(null=True, blank=True, unique=True)
    badge_issued_at = models.DateTimeField(null=True, blank=True)
    valid_from = models.DateField(null=True, blank=True)
    valid_to = models.DateField(null=True, blank=True)
    revocation_reason = models.TextField(blank=True)
    revoked_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "training_accred_media_accreditations"
        managed = True

    def __str__(self) -> str:
        return f"{self.applicant_name} ({self.status})"


class BadgeRevocationEntry(models.Model):
    REASON_CHOICES = [
        ("BREACH", "Breach"),
        ("MISCONDUCT", "Misconduct"),
        ("SECURITY", "Security"),
        ("EXPIRED", "Expired"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    badge_serial = models.UUIDField()
    reason_code = models.CharField(max_length=30, choices=REASON_CHOICES)
    revoked_at = models.DateTimeField(auto_now_add=True)
    officer_id = models.UUIDField()

    class Meta:
        db_table = "training_accred_badge_revocations"
        managed = False

    def __str__(self) -> str:
        return f"BadgeRevocation({self.badge_serial})"


class AccreditationAuditEntry(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    action = models.CharField(max_length=100)
    actor_id = models.UUIDField()
    accreditation_id = models.UUIDField()
    timestamp = models.DateTimeField(auto_now_add=True)
    ip = models.GenericIPAddressField(null=True, blank=True)
    purpose_code = models.CharField(max_length=50)
    merkle_node_id = models.CharField(max_length=100, blank=True)

    class Meta:
        db_table = "training_accred_audit_entries"
        managed = True

    def __str__(self) -> str:
        return f"Audit({self.action}, {self.actor_id})"


class ObserverOrganisation(models.Model):
    ORG_TYPE_CHOICES = [
        ("DOMESTIC_NGO", "Domestic NGO"),
        ("INTERNATIONAL_MISSION", "International Mission"),
        ("DIPLOMATIC_MISSION", "Diplomatic Mission"),
        ("RELIGIOUS_BODY", "Religious Body"),
        ("ACADEMIC", "Academic"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    name = models.CharField(max_length=500)
    org_type = models.CharField(max_length=30, choices=ORG_TYPE_CHOICES)
    jurisdiction = models.CharField(max_length=100)
    mou_required = models.BooleanField(default=False)
    mou_signed_at = models.DateTimeField(null=True, blank=True)
    approved_at = models.DateTimeField(null=True, blank=True)
    observer_quota = models.PositiveIntegerField(default=0)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "training_accred_observer_orgs"
        managed = True

    def __str__(self) -> str:
        return self.name


class IndividualObserver(models.Model):
    STATUS_CHOICES = [
        ("SUBMITTED", "Submitted"),
        ("DOCS_VERIFIED", "Docs Verified"),
        ("BACKGROUND_CHECK", "Background Check"),
        ("APPROVED", "Approved"),
        ("REJECTED", "Rejected"),
        ("CREDENTIAL_ISSUED", "Credential Issued"),
        ("DEPLOYED", "Deployed"),
        ("SUSPENDED", "Suspended"),
        ("REVOKED", "Revoked"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    organisation = models.ForeignKey(
        ObserverOrganisation,
        on_delete=models.CASCADE,
        related_name="observers",
    )
    full_name = models.CharField(max_length=500)
    id_number = models.CharField(max_length=50)
    nationality = models.CharField(max_length=100)
    photo_url = models.URLField(blank=True)
    deployment_county_codes = models.JSONField(default=list)
    assigned_constituency_codes = models.JSONField(default=list)
    status = models.CharField(
        max_length=30, choices=STATUS_CHOICES, default="SUBMITTED"
    )
    credential_serial = models.UUIDField(null=True, blank=True, unique=True)
    credential_issued_at = models.DateTimeField(null=True, blank=True)
    valid_from = models.DateField(null=True, blank=True)
    valid_to = models.DateField(null=True, blank=True)
    revocation_reason = models.TextField(blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "training_accred_observers"
        managed = True

    def __str__(self) -> str:
        return f"{self.full_name} ({self.status})"


class CredentialRevocationEntry(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    credential_serial = models.UUIDField()
    reason = models.TextField()
    revoked_at = models.DateTimeField(auto_now_add=True)
    officer_id = models.UUIDField()

    class Meta:
        db_table = "training_accred_credential_revocations"
        managed = True

    def __str__(self) -> str:
        return f"CredentialRevocation({self.credential_serial})"
