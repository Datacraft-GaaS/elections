"""Django models for declaration-admin (WP-3.3, M51, M63).

These are read-models backed by the Rust event-sourced projections.
The authoritative store is the event log in rts-svc; these rows are
projection copies for the Django admin UI.  No business logic lives here.
"""

import uuid

from django.db import models


class Declaration(models.Model):
    """Projection of the Declaration aggregate."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    contest_id = models.CharField(max_length=128)
    election_id = models.CharField(max_length=128)
    status = models.CharField(
        max_length=32,
        choices=[
            ("pending", "Pending"),
            ("declared", "Declared"),
            ("runoff_required", "Runoff Required"),
            ("tie_pending", "Tie Pending"),
            ("amended", "Amended"),
            ("annulled", "Annulled"),
        ],
        default="pending",
    )
    winner_id = models.CharField(max_length=128, blank=True, null=True)
    declared_at = models.DateTimeField(blank=True, null=True)
    declared_by = models.CharField(max_length=128, blank=True, null=True)
    ceremony_id = models.CharField(max_length=128, blank=True, null=True)
    tally_snapshot_id = models.CharField(max_length=256, blank=True, null=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "declaration_admin_declarations"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"Declaration({self.contest_id}, {self.status})"


class CertificateOfElection(models.Model):
    """Projection of CertificateOfElection aggregate."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    declaration = models.ForeignKey(
        Declaration, on_delete=models.PROTECT, related_name="certificates"
    )
    winner_id = models.CharField(max_length=128)
    status = models.CharField(
        max_length=16,
        choices=[
            ("draft", "Draft"),
            ("issued", "Issued"),
            ("superseded", "Superseded"),
            ("revoked", "Revoked"),
        ],
        default="draft",
    )
    issued_at = models.DateTimeField(blank=True, null=True)
    pdf_minio_ref = models.CharField(max_length=512, blank=True, null=True)
    tlog_leaf_index = models.BigIntegerField(blank=True, null=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "declaration_admin_certificates"

    def __str__(self) -> str:
        return f"Certificate({self.declaration_id}, {self.status})"


class PetitionWindow(models.Model):
    """Projection of PetitionWindowTracker aggregate."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    declaration = models.OneToOneField(
        Declaration, on_delete=models.PROTECT, related_name="petition_window"
    )
    status = models.CharField(
        max_length=16,
        choices=[
            ("unopened", "Unopened"),
            ("open", "Open"),
            ("closed", "Closed"),
            ("petitioned", "Petitioned"),
        ],
        default="unopened",
    )
    # T+28d 23:59:59 EAT inclusive (EC-TIM-04, EC-TIM-08); stored UTC.
    window_open_at = models.DateTimeField(blank=True, null=True)
    window_close_at = models.DateTimeField(blank=True, null=True)
    petition_court_case = models.CharField(max_length=128, blank=True, null=True)
    judiciary_ack_at = models.DateTimeField(blank=True, null=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "declaration_admin_petition_windows"

    def __str__(self) -> str:
        return f"PetitionWindow({self.declaration_id}, {self.status})"


class DeclarationAmendment(models.Model):
    """Audit record for each amendment event (retained 7 years, no soft delete)."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    declaration = models.ForeignKey(
        Declaration, on_delete=models.PROTECT, related_name="amendments"
    )
    reason = models.TextField()
    amended_by = models.CharField(max_length=128)
    approved_by = models.CharField(max_length=128)
    amended_at = models.DateTimeField()
    previous_snapshot_hash = models.CharField(max_length=128)
    new_snapshot_hash = models.CharField(max_length=128)
    court_order_ref = models.CharField(max_length=128, blank=True, null=True)
    evidence_file_ref = models.CharField(max_length=512, blank=True, null=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "declaration_admin_amendments"
        ordering = ["amended_at"]

    def __str__(self) -> str:
        return f"Amendment({self.declaration_id}, {self.amended_at})"


# ---- Ballot-serial reconciliation models ------------------------------------


class ReconciliationHold(models.Model):
    """Read-projection of ballot-serial discrepancy holds (M63)."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    station_code = models.CharField(max_length=32, db_index=True)
    reconciliation_id = models.UUIDField()
    flag_reason = models.TextField()
    raised_at = models.DateTimeField()
    resolved_at = models.DateTimeField(blank=True, null=True)
    resolved_by_po = models.CharField(max_length=128, blank=True, null=True)
    resolved_by_cro = models.CharField(max_length=128, blank=True, null=True)
    resolve_reason = models.TextField(blank=True, null=True)
    # Blocks M10 result publication until cleared (FR-M63-025).
    result_hold_active = models.BooleanField(default=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "declaration_admin_reconciliation_holds"
        ordering = ["-raised_at"]

    def __str__(self) -> str:
        return f"Hold({self.station_code}, active={self.result_hold_active})"
