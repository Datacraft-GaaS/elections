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

"""Read-only models for the RFID ballot-box cover custody tables.

Schema is owned by V051__rfid_ballot_covers.sql; `managed = False` keeps Django
out of the DDL.

The one thing here that is not a straight column-for-column mirror is
`cover_state`. V051 stores no such column: a cover's state *is* the `to_state`
of the last row in its `eel_cover_state_transitions` chain, which is what makes
the chain the system of record. The models used to declare `cover_state` as a
`CharField`, so every read raised `column ... does not exist`, and the fix is to
derive it rather than to add the column -- adding it would give the state two
homes and let the chain and the tag disagree.
"""

from django.db import models
from django.db.models import OuterRef, Subquery


def as_bytes(value: object) -> bytes:
    """Read a BYTEA column's value as `bytes`.

    Django's stubs type a `BinaryField` *attribute* as the field class rather
    than as the value it holds, so `bytes(row.epc)` is correct at runtime and a
    type error to a checker. The conversion is needed at every read of `epc`,
    `prev_hash`, `this_hash`, `tag_challenge` and `tag_response`, so it is
    narrowed here once instead of at each call site.

    `memoryview` is accepted as well as `bytes`, because psycopg returns BYTEA
    as either depending on the driver path.
    """
    if isinstance(value, (bytes, memoryview)):
        return bytes(value)
    raise TypeError(f"expected bytes from a BYTEA column, got {type(value).__name__}")


class RfidCoverTag(models.Model):
    """One row per manufactured RFID cover tag (eel_rfid_cover_tags)."""

    class ChipFamily(models.TextChoices):
        NXP_UCODE9 = "nxp_ucode9", "NXP UCODE 9"
        IMPINJ_MONZA_R6P = "impinj_monza_r6p", "Impinj Monza R6-P"

    class TamperState(models.TextChoices):
        # V051's CHECK admits exactly these five. The model previously listed
        # intact/lifted/quarantine/retired, two of which ('lifted', 'retired')
        # no row can hold and three of which ('cover_lifted', 'antenna_broken',
        # 'clone_detected', 'killed') it could not express.
        INTACT = "intact", "Intact"
        COVER_LIFTED = "cover_lifted", "Cover Lifted"
        ANTENNA_BROKEN = "antenna_broken", "Antenna Broken"
        CLONE_DETECTED = "clone_detected", "Clone Detected"
        KILLED = "killed", "Killed"

    tag_id = models.UUIDField(primary_key=True)
    # BYTEA in V051, not a hex string: the EPC is a 96-bit binary identifier and
    # the column is UNIQUE on those bytes.
    epc = models.BinaryField(unique=True)
    manufacturer = models.TextField()
    # Deferred FK to eel_printing_lots, which lives in another service's schema
    # (M39). Kept as a bare UUID rather than a ForeignKey so this app does not
    # depend on a table it cannot see.
    lot_id = models.UUIDField()
    chip_family = models.TextField(choices=ChipFamily.choices)
    tag_secret_hsm_key_id = models.TextField()
    issued_at = models.DateTimeField()
    paired_with_box = models.UUIDField(null=True, blank=True)
    paired_at = models.DateTimeField(null=True, blank=True)
    tamper_state = models.TextField(
        choices=TamperState.choices, default=TamperState.INTACT
    )
    tamper_detected_at = models.DateTimeField(null=True, blank=True)
    tamper_detected_by = models.TextField(null=True, blank=True)
    retired_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_rfid_cover_tags"
        verbose_name = "RFID Cover Tag"
        verbose_name_plural = "RFID Cover Tags"
        ordering = ["-issued_at"]

    @classmethod
    def with_current_state(cls):
        """Queryset annotated with `cover_state`, the chain's latest `to_state`.

        A correlated subquery rather than a property, because the admin renders
        this in a list: a property would issue one query per row. Ordered by
        `transitioned_at DESC, transition_id DESC` -- the tiebreak matters, since
        two transitions can share a timestamp and the chain's order is then only
        recoverable from the id.
        """
        latest = (
            CoverStateTransition.objects.filter(tag_id=OuterRef("tag_id"))
            .order_by("-transitioned_at", "-transition_id")
            .values("to_state")[:1]
        )
        return cls.objects.annotate(cover_state=Subquery(latest))

    @property
    def epc_hex(self) -> str:
        """The EPC as uppercase hex.

        `epc` is BYTEA, so rendering the field directly gives `b'\\x30...'`.
        Templates that show an EPC want this.
        """
        return as_bytes(self.epc).hex().upper()

    @property
    def current_state(self) -> str | None:
        """The chain's latest `to_state`, for a single tag.

        One query per tag. A list should use `with_current_state()` instead,
        which annotates the same value as a correlated subquery.
        """
        return (
            self.transitions.order_by("-transitioned_at", "-transition_id")
            .values_list("to_state", flat=True)
            .first()
        )

    def __str__(self) -> str:
        return f"{self.epc_hex} ({self.tamper_state})"


class RfidCoverRead(models.Model):
    """Every read at a reader gate, kit, or handheld (eel_rfid_cover_reads)."""

    class ReaderType(models.TextChoices):
        # V051's CHECK admits eight reader types; the model listed five, so
        # station_arrival, airline_cargo and mfa_customs reads could not be
        # represented and two of the five it did list (kit_handheld,
        # ro_desktop) it listed correctly while omitting their siblings.
        WAREHOUSE_GATE = "warehouse_gate", "Warehouse Gate"
        TRANSIT_CHECKPOINT = "transit_checkpoint", "Transit Checkpoint"
        STATION_ARRIVAL = "station_arrival", "Station Arrival"
        KIT_HANDHELD = "kit_handheld", "Kit Handheld"
        RO_DESKTOP = "ro_desktop", "RO Desktop"
        ARCHIVE_GATE = "archive_gate", "Archive Gate"
        AIRLINE_CARGO = "airline_cargo", "Airline Cargo"
        MFA_CUSTOMS = "mfa_customs", "MFA Customs"

    read_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag,
        on_delete=models.DO_NOTHING,
        db_column="tag_id",
        to_field="tag_id",
        related_name="reads",
    )
    # V051 calls this `reader_id`. The model called it `reader_serial` and the
    # reader-health view groups by it, so the whole screen queried a column that
    # does not exist.
    reader_id = models.TextField()
    reader_type = models.TextField(choices=ReaderType.choices)
    read_at = models.DateTimeField()
    geo_lat = models.FloatField(null=True, blank=True)
    geo_lon = models.FloatField(null=True, blank=True)
    geo_accuracy_m = models.FloatField(null=True, blank=True)
    # The HMAC challenge exchange is stored as the two raw blobs. The model had
    # `challenge_nonce`/`challenge_response` as hex CharFields, neither of which
    # is a column here.
    tag_challenge = models.BinaryField(null=True, blank=True)
    tag_response = models.BinaryField(null=True, blank=True)
    challenge_valid = models.BooleanField(default=False)
    tamper_indicator = models.BooleanField(default=False)
    reader_signature = models.BinaryField(null=True, blank=True)
    custody_event_id = models.UUIDField(null=True, blank=True)
    correlation_id = models.UUIDField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_rfid_cover_reads"
        verbose_name = "Cover Read"
        verbose_name_plural = "Cover Reads"
        ordering = ["-read_at"]

    def __str__(self) -> str:
        return f"Read {self.read_id} tag={self.tag_id} at={self.read_at}"


class CoverStateTransition(models.Model):
    """Append-only FSM transitions per tag (eel_cover_state_transitions).

    V051 installs a BEFORE UPDATE/DELETE trigger that raises unconditionally, so
    these rows are immutable and the per-tag BLAKE3 chain is the system of record.
    """

    class CoverState(models.TextChoices):
        MANUFACTURED = "MANUFACTURED", "Manufactured"
        PRINTER = "PRINTER", "Printer"
        WAREHOUSE = "WAREHOUSE", "Warehouse"
        DISPATCHED = "DISPATCHED", "Dispatched"
        IN_TRANSIT = "IN_TRANSIT", "In Transit"
        STATION_RECEIVED = "STATION_RECEIVED", "Station Received"
        STATION_OPENED = "STATION_OPENED", "Station Opened"
        VOTING_ACTIVE = "VOTING_ACTIVE", "Voting Active"
        STATION_CLOSED = "STATION_CLOSED", "Station Closed"
        SEALED_RETURN = "SEALED_RETURN", "Sealed Return"
        TALLY_ARRIVED = "TALLY_ARRIVED", "Tally Arrived"
        COUNT_COMPLETE = "COUNT_COMPLETE", "Count Complete"
        STORAGE_ARCHIVE = "STORAGE_ARCHIVE", "Storage Archive"
        DESTROYED = "DESTROYED", "Destroyed"
        QUARANTINE = "QUARANTINE", "Quarantine"

    transition_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag,
        on_delete=models.DO_NOTHING,
        db_column="tag_id",
        to_field="tag_id",
        related_name="transitions",
    )
    from_state = models.TextField()
    # Only `to_state` is CHECK-constrained in V051; `from_state` is free text.
    to_state = models.TextField(choices=CoverState.choices)
    # V051 names this `triggered_by_read`; the model called it `read_id` via
    # `db_column`, which is the name of the *other* table's key.
    triggered_by_read = models.ForeignKey(
        RfidCoverRead,
        on_delete=models.DO_NOTHING,
        db_column="triggered_by_read",
        to_field="read_id",
        null=True,
        blank=True,
        related_name="triggered_transitions",
    )
    officer_id = models.UUIDField(null=True, blank=True)
    officer_svid = models.TextField(null=True, blank=True)
    # V051 stores the officer's signature as BYTEA; the model had a TextField
    # named `hsm_signature` and a separate `merkle_anchor_id` CharField. Neither
    # column exists -- the transparency-log anchor for a transition is derivable
    # from the chain, and is not stored per row.
    officer_signature = models.BinaryField(null=True, blank=True)
    prev_hash = models.BinaryField(null=True, blank=True)
    this_hash = models.BinaryField()
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
    """Cover-lift or mismatch alert (eel_cover_tamper_alerts).

    V051 installs a BEFORE UPDATE/DELETE trigger on this table too, which
    contradicts its own COMMENT ("tracks ECC dispatch and resolution"): the
    dispatch and resolution columns exist but no UPDATE may set them. The model
    mirrors the columns as they are; see the note on the admin, which used to
    offer a bulk-resolve action the trigger refuses.
    """

    class Severity(models.TextChoices):
        # V051's CHECK admits info/warn/critical. The model listed
        # low/medium/high/critical -- four values of which none is admissible.
        INFO = "info", "Info"
        WARN = "warn", "Warn"
        CRITICAL = "critical", "Critical"

    alert_id = models.UUIDField(primary_key=True)
    tag = models.ForeignKey(
        RfidCoverTag,
        on_delete=models.DO_NOTHING,
        db_column="tag_id",
        to_field="tag_id",
        related_name="alerts",
    )
    detected_at = models.DateTimeField()
    # `triggering_read`, not `trigger_read_id`.
    triggering_read = models.ForeignKey(
        RfidCoverRead,
        on_delete=models.DO_NOTHING,
        db_column="triggering_read",
        to_field="read_id",
        null=True,
        blank=True,
        related_name="triggered_alerts",
    )
    expected_state = models.TextField(null=True, blank=True)
    observed_state = models.TextField(null=True, blank=True)
    severity = models.TextField(choices=Severity.choices, default=Severity.WARN)
    ecc_dispatched = models.BooleanField(default=False)
    ecc_dispatched_at = models.DateTimeField(null=True, blank=True)
    resolved_at = models.DateTimeField(null=True, blank=True)
    resolution_note = models.TextField(null=True, blank=True)

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
