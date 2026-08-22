"""
Read-only Django models for RFID ballot box cover tags.

Schema is owned by Rust migration V051__rfid_ballot_covers.sql.
managed=False prevents Django from touching the tables.
"""

from django.db import models


class RfidCoverTag(models.Model):
    """One row per manufactured RFID cover tag (eel_rfid_cover_tags)."""

    class ChipFamily(models.TextChoices):
        NXP_UCODE9 = "nxp_ucode9", "NXP UCODE 9"
        IMPINJ_MONZA_R6P = "impinj_monza_r6p", "Impinj Monza R6-P"

    class TamperState(models.TextChoices):
        INTACT = "intact", "Intact"
        LIFTED = "lifted", "Lifted"
        QUARANTINE = "quarantine", "Quarantine"
        RETIRED = "retired", "Retired"

    tag_id = models.UUIDField(primary_key=True)
    epc = models.CharField(
        max_length=24, unique=True, help_text="96-bit EPC hex string"
    )
    chip_family = models.CharField(max_length=32, choices=ChipFamily.choices)
    paired_with_box = models.CharField(
        max_length=64,
        blank=True,
        null=True,
        help_text="Box body serial this tag is bound to",
    )
    tamper_state = models.CharField(
        max_length=16, choices=TamperState.choices, default=TamperState.INTACT
    )
    cover_state = models.CharField(
        max_length=32, help_text="FSM state e.g. MANUFACTURED, WAREHOUSE"
    )
    manufacturer = models.CharField(max_length=128)
    issued_at = models.DateTimeField()
    hsm_key_ref = models.CharField(
        max_length=128, help_text="HSM slot reference for per-tag HMAC secret"
    )
    bluetooth_id = models.CharField(max_length=64, blank=True, null=True)
    notes = models.TextField(blank=True)

    class Meta:
        managed = False
        db_table = "eel_rfid_cover_tags"
        verbose_name = "RFID Cover Tag"
        verbose_name_plural = "RFID Cover Tags"
        ordering = ["-issued_at"]

    def __str__(self) -> str:
        return f"{self.epc} ({self.cover_state})"


class RfidCoverRead(models.Model):
    """Every read at a reader gate, kit, or handheld (eel_rfid_cover_reads)."""

    class ReaderType(models.TextChoices):
        WAREHOUSE_GATE = "warehouse_gate", "Warehouse Gate"
        TRANSIT_CHECKPOINT = "transit_checkpoint", "Transit Checkpoint"
        KIT_HANDHELD = "kit_handheld", "Kit Handheld"
        RO_DESKTOP = "ro_desktop", "RO Desktop"
        ARCHIVE_GATE = "archive_gate", "Archive Gate"

    read_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag, on_delete=models.DO_NOTHING, db_column="tag_id", to_field="tag_id"
    )
    reader_serial = models.CharField(max_length=64)
    reader_type = models.CharField(max_length=32, choices=ReaderType.choices)
    challenge_nonce = models.CharField(max_length=32, help_text="16-byte nonce hex")
    challenge_response = models.CharField(
        max_length=64, help_text="HMAC-SHA256 response hex"
    )
    challenge_valid = models.BooleanField(default=False)
    tamper_indicator = models.BooleanField(
        default=False, help_text="Tag reported tamper bit or failed to respond"
    )
    rssi = models.FloatField(null=True, blank=True)
    latitude = models.DecimalField(
        max_digits=10, decimal_places=7, null=True, blank=True
    )
    longitude = models.DecimalField(
        max_digits=10, decimal_places=7, null=True, blank=True
    )
    reader_signature = models.TextField(help_text="Reader SVID-signed read blob")
    read_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "eel_rfid_cover_reads"
        verbose_name = "Cover Read"
        verbose_name_plural = "Cover Reads"
        ordering = ["-read_at"]

    def __str__(self) -> str:
        return f"Read {self.read_id} tag={self.tag_id} at={self.read_at}"


class CoverStateTransition(models.Model):
    """Append-only FSM transitions per tag (eel_cover_state_transitions)."""

    transition_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag,
        on_delete=models.DO_NOTHING,
        db_column="tag_id",
        to_field="tag_id",
        related_name="transitions",
    )
    from_state = models.CharField(max_length=32)
    to_state = models.CharField(max_length=32)
    officer_id = models.CharField(max_length=64)
    reader_read = models.ForeignKey(
        RfidCoverRead,
        on_delete=models.DO_NOTHING,
        db_column="read_id",
        to_field="read_id",
        null=True,
        blank=True,
    )
    prev_hash = models.CharField(
        max_length=64, help_text="SHA-256 of previous transition row"
    )
    this_hash = models.CharField(
        max_length=64, help_text="SHA-256 of this transition row"
    )
    hsm_signature = models.TextField(help_text="Officer HSM signature over this_hash")
    merkle_anchor_id = models.CharField(
        max_length=128, blank=True, null=True, help_text="P05 transparency log anchor"
    )
    transitioned_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "eel_cover_state_transitions"
        verbose_name = "State Transition"
        verbose_name_plural = "State Transitions"
        ordering = ["transitioned_at"]

    def __str__(self) -> str:
        return f"{self.tag_id}: {self.from_state} -> {self.to_state}"


class CoverTamperAlert(models.Model):
    """Cover-lift or mismatch alert (eel_cover_tamper_alerts)."""

    class Severity(models.TextChoices):
        LOW = "low", "Low"
        MEDIUM = "medium", "Medium"
        HIGH = "high", "High"
        CRITICAL = "critical", "Critical"

    alert_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag,
        on_delete=models.DO_NOTHING,
        db_column="tag_id",
        to_field="tag_id",
        related_name="alerts",
    )
    trigger_read = models.ForeignKey(
        RfidCoverRead,
        on_delete=models.DO_NOTHING,
        db_column="trigger_read_id",
        to_field="read_id",
        null=True,
        blank=True,
    )
    severity = models.CharField(max_length=16, choices=Severity.choices)
    description = models.TextField()
    geo_lat = models.DecimalField(
        max_digits=10, decimal_places=7, null=True, blank=True
    )
    geo_lon = models.DecimalField(
        max_digits=10, decimal_places=7, null=True, blank=True
    )
    expected_state = models.CharField(max_length=32)
    observed_state = models.CharField(max_length=32)
    ecc_dispatched = models.BooleanField(default=False)
    dispatch_officer_id = models.CharField(max_length=64, blank=True, null=True)
    resolved_at = models.DateTimeField(null=True, blank=True)
    resolution_note = models.TextField(blank=True)
    detected_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "eel_cover_tamper_alerts"
        verbose_name = "Tamper Alert"
        verbose_name_plural = "Tamper Alerts"
        ordering = ["-detected_at"]

    def __str__(self) -> str:
        return f"Alert {self.alert_id} [{self.severity}] tag={self.tag_id}"

    @property
    def is_resolved(self) -> bool:
        return self.resolved_at is not None
