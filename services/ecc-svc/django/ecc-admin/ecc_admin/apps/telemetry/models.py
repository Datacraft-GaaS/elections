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

"""Read-only mirrors of the tables V052__mobile_telemetry.sql creates.

Schema is owned by that migration; `managed = False` keeps Django out of the DDL.

The set below is exactly V052's, which is not the set this module used to carry.
It had three models for tables no migration creates --
`eel_telemetry_device_snapshots`, `eel_telemetry_officer_sessions` and
`eel_telemetry_merkle_snapshots` -- and no models for three tables V052 does
create (`eel_telemetry_sessions`, `eel_telemetry_activity_frames`,
`eel_telemetry_geo_dwells`). The first three could not be queried in any
environment, because nothing ever created the tables; the last three were
invisible to the console that exists to read them. V052's actual snapshots table
is `eel_telemetry_snapshots`, whose shape is the union of the two invented
snapshot models' columns.

Two spellings in this file are load-bearing rather than stylistic:

- `class` is the column name, so the field is `signal_class` with
  `db_column="class"` (`class` is a Python keyword).
- The signal class values are the *singular* ones. V052's CHECK admits
  `movement`, `read`, `write`, `lock_unlock`, `drop`, and the ingest crate writes
  those. An earlier version of this module used the plural forms the query DSL
  uses, which no row can hold.
"""

from django.db import models


class RealField(models.FloatField):
    """A FloatField that reports itself as `real`.

    Django ships no single-precision float field. `accel_x_avg` and the rest of
    the activity-frame columns are REAL, so a FloatField against them is
    permanent drift -- `makemigrations` would emit an ALTER widening them to
    double precision.
    """

    def db_type(self, connection) -> str:
        return "real"


class TelemetryDevice(models.Model):
    """A registered field device (V052 table 1)."""

    class PersonaRole(models.TextChoices):
        PRESIDING_OFFICER = "presiding_officer", "Presiding Officer"
        DEPUTY = "deputy", "Deputy"
        OBSERVER = "observer", "Observer"
        AUDITOR = "auditor", "Auditor"

    device_id = models.UUIDField(primary_key=True)
    kit_id = models.TextField()
    app_id = models.TextField()
    persona_role = models.TextField(choices=PersonaRole.choices)
    spiffe_id = models.TextField(null=True, blank=True)
    hmac_key_hsm_ref = models.TextField()
    first_seen_at = models.DateTimeField()
    last_seen_at = models.DateTimeField()
    revoked_at = models.DateTimeField(null=True, blank=True)
    revocation_reason = models.TextField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_devices"
        verbose_name = "Telemetry Device"
        verbose_name_plural = "Telemetry Devices"
        ordering = ["-last_seen_at"]

    def __str__(self) -> str:
        return f"{self.device_id} ({self.persona_role})"


class TelemetrySession(models.Model):
    """An operator session window on a device (V052 table 2)."""

    class ExitReason(models.TextChoices):
        NORMAL = "normal", "Normal"
        TIMEOUT = "timeout", "Timeout"
        REVOKED = "revoked", "Revoked"
        DURESS = "duress", "Duress"
        CRASH = "crash", "Crash"

    session_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    actor_svid = models.TextField(db_index=True)
    entered_at = models.DateTimeField()
    exited_at = models.DateTimeField(null=True, blank=True)
    exit_reason = models.TextField(choices=ExitReason.choices, null=True, blank=True)
    signals_count = models.BigIntegerField(default=0)

    class Meta:
        managed = False
        db_table = "eel_telemetry_sessions"
        verbose_name = "Telemetry Session"
        verbose_name_plural = "Telemetry Sessions"
        ordering = ["-entered_at"]

    def __str__(self) -> str:
        return f"Session {self.session_id} on {self.device_id}"


class TelemetrySignal(models.Model):
    """One signal row from eel_telemetry_signals (partitioned by occurred_at)."""

    class SignalClass(models.TextChoices):
        # The singular spellings V052's CHECK admits. `movement` not
        # `movements`, `read` not `reads`: these are the values the ingest crate
        # writes, and a filter value that cannot appear in a row is a filter
        # that always returns nothing.
        LOCATION = "location", "Location"
        TIME = "time", "Time"
        ACTIVITY = "activity", "Activity"
        PROXIMAL_RFID = "proximal_rfid", "Proximal RFID"
        MOVEMENT = "movement", "Movement"
        LOGIN_LOGOUT = "login_logout", "Login / Logout"
        READ = "read", "Read"
        WRITE = "write", "Write"
        BATTERY = "battery", "Battery"
        LOCK_UNLOCK = "lock_unlock", "Lock / Unlock"
        DROP = "drop", "Drop"
        NETWORK = "network", "Network"
        ATTESTATION = "attestation", "Attestation"

    # The table's key is (signal_id, occurred_at) -- the partition key has to be
    # in the primary key for a unique constraint to be valid on a partitioned
    # table. A single-column `signal_id = UUIDField(primary_key=True)` here
    # would claim a uniqueness the table does not have.
    pk = models.CompositePrimaryKey("signal_id", "occurred_at")
    signal_id = models.UUIDField()
    device_id = models.UUIDField(db_index=True)
    session_id = models.UUIDField(null=True, blank=True)
    signal_class = models.TextField(
        db_column="class", choices=SignalClass.choices, db_index=True
    )
    payload = models.JSONField()
    occurred_at = models.DateTimeField(db_index=True)
    ingested_at = models.DateTimeField()
    monotonic_counter = models.BigIntegerField()
    prev_hash = models.BinaryField(null=True, blank=True)
    this_hash = models.BinaryField()
    hmac_signature = models.BinaryField()
    server_signature = models.BinaryField(null=True, blank=True)
    trace_id = models.BinaryField(null=True, blank=True)
    span_id = models.BinaryField(null=True, blank=True)
    correlation_id = models.UUIDField(null=True, blank=True)
    causation_id = models.UUIDField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_signals"
        verbose_name = "Telemetry Signal"
        verbose_name_plural = "Telemetry Signals"
        ordering = ["-occurred_at"]

    def __str__(self) -> str:
        return f"{self.signal_class} @ {self.occurred_at} [{self.device_id}]"


class TelemetryActivityFrame(models.Model):
    """A one-second accelerometer/gyro aggregate (V052 table 4)."""

    frame_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    second_bucket = models.DateTimeField(db_index=True)
    accel_x_avg = RealField()
    accel_y_avg = RealField()
    accel_z_avg = RealField()
    accel_max_g = RealField()
    gyro_x_avg = RealField()
    gyro_y_avg = RealField()
    gyro_z_avg = RealField()
    motion_class = models.TextField()
    battery_pct = models.SmallIntegerField(null=True, blank=True)
    battery_temp_c = RealField(null=True, blank=True)
    network_class = models.TextField(null=True, blank=True)
    lat_coarse = RealField(null=True, blank=True)
    lon_coarse = RealField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_activity_frames"
        verbose_name = "Activity Frame"
        verbose_name_plural = "Activity Frames"
        ordering = ["-second_bucket"]

    def __str__(self) -> str:
        return f"{self.motion_class} @ {self.second_bucket} [{self.device_id}]"


class TelemetryGeoDwell(models.Model):
    """A dwell interval inside a named polygon (V052 table 5)."""

    dwell_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    polygon_ref = models.TextField()
    entered_at = models.DateTimeField(db_index=True)
    exited_at = models.DateTimeField(null=True, blank=True)
    dwell_seconds = models.IntegerField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_geo_dwells"
        verbose_name = "Geo Dwell"
        verbose_name_plural = "Geo Dwells"
        ordering = ["-entered_at"]

    def __str__(self) -> str:
        return f"{self.polygon_ref} @ {self.entered_at}"


class TelemetryAlert(models.Model):
    """Anomaly / alert record from eel_telemetry_alerts (V052 table 6)."""

    class AlertClass(models.TextChoices):
        DROP = "drop", "Drop"
        GEO_FENCE_BREACH = "geo_fence_breach", "Geo-fence Breach"
        SESSION_HIJACK = "session_hijack", "Session Hijack"
        BATTERY_LOW = "battery_low", "Battery Low"
        RFID_ABSENCE = "rfid_absence", "RFID Absence"
        LOGIN_TIME_ANOMALY = "login_time_anomaly", "Login-time Anomaly"
        READ_HEAVY = "read_heavy", "Read-heavy"
        COORDINATED_ANOMALY = "coordinated_anomaly", "Coordinated Anomaly"
        CHASSIS_TAMPER = "chassis_tamper", "Chassis Tamper"

    class Severity(models.TextChoices):
        INFO = "info", "Info"
        WARN = "warn", "Warn"
        CRITICAL = "critical", "Critical"

    alert_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    alert_class = models.TextField(
        db_column="class", choices=AlertClass.choices, db_index=True
    )
    severity = models.TextField(choices=Severity.choices, db_index=True)
    detected_at = models.DateTimeField(db_index=True)
    triggering_signal = models.UUIDField(null=True, blank=True)
    payload = models.JSONField(default=dict)
    ecc_dispatched = models.BooleanField(default=False)
    ecc_dispatched_at = models.DateTimeField(null=True, blank=True)
    resolved_at = models.DateTimeField(null=True, blank=True)
    resolution_note = models.TextField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_alerts"
        verbose_name = "Telemetry Alert"
        verbose_name_plural = "Telemetry Alerts"
        ordering = ["-detected_at"]

    def __str__(self) -> str:
        return f"{self.severity.upper()} {self.alert_class} @ {self.detected_at}"


class TelemetryQueryAudit(models.Model):
    """Audit log of admin DSL queries (V052 table 7)."""

    query_id = models.UUIDField(primary_key=True)
    actor_svid = models.TextField(db_index=True)
    actor_role = models.TextField(null=True, blank=True)
    query_dsl = models.JSONField(null=True, blank=True)
    query_text = models.TextField(null=True, blank=True)
    executed_at = models.DateTimeField(db_index=True)
    elapsed_ms = models.IntegerField(null=True, blank=True)
    row_count = models.BigIntegerField(null=True, blank=True)
    evidence_bundle_hash = models.BinaryField(null=True, blank=True)
    evidence_bundle_ref = models.TextField(null=True, blank=True)
    correlation_id = models.UUIDField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_queries"
        verbose_name = "Query Audit"
        verbose_name_plural = "Query Audit Log"
        ordering = ["-executed_at"]

    def __str__(self) -> str:
        return f"Query {self.query_id} by {self.actor_svid} @ {self.executed_at}"


class TelemetrySnapshot(models.Model):
    """A daily per-device Merkle root anchored to the transparency log (V052 table 8).

    One row per (snapshot_date, device_id); `device_id` is nullable for the
    whole-register root of a day.
    """

    snapshot_id = models.UUIDField(primary_key=True)
    snapshot_date = models.DateField(db_index=True)
    device_id = models.UUIDField(null=True, blank=True)
    device_merkle_root = models.BinaryField(null=True, blank=True)
    snapshot_taken_at = models.DateTimeField()
    tlog_leaf_index = models.BigIntegerField(null=True, blank=True)
    tlog_root_hash = models.BinaryField(null=True, blank=True)
    inclusion_proof = models.BinaryField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_snapshots"
        verbose_name = "Telemetry Snapshot"
        verbose_name_plural = "Telemetry Snapshots"
        ordering = ["-snapshot_date"]

    def __str__(self) -> str:
        return f"Snapshot {self.snapshot_date} [{self.device_id or 'all devices'}]"
