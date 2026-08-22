"""
Read-only Django models mirroring the 8 V052 telemetry tables.

Schema is owned by Rust migration V052__mobile_telemetry.sql.
managed=False prevents Django from creating or modifying the tables.
"""

from django.db import models


class TelemetrySignal(models.Model):
    """One signal row from eel_telemetry_signals (partitioned by month)."""

    class SignalClass(models.TextChoices):
        LOCATION = "location", "Location"
        TIME = "time", "Time"
        ACTIVITY = "activity", "Activity"
        PROXIMAL_RFID = "proximal_rfid", "Proximal RFID"
        MOVEMENTS = "movements", "Movements"
        LOGIN_LOGOUT = "login_logout", "Login / Logout"
        READS = "reads", "Reads"
        WRITES = "writes", "Writes"
        BATTERY = "battery", "Battery"
        LOCKS_UNLOCKS = "locks_unlocks", "Locks / Unlocks"
        PHYSICAL_DROPS = "physical_drops", "Physical Drops"
        NETWORK = "network", "Network"
        ATTESTATION = "attestation", "Attestation"

    signal_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    signal_class = models.CharField(max_length=32, choices=SignalClass.choices)
    occurred_at = models.DateTimeField(db_index=True)
    payload = models.JSONField()
    geo_lat = models.FloatField(null=True, blank=True)
    geo_lon = models.FloatField(null=True, blank=True)
    severity = models.CharField(max_length=16, blank=True)
    station_id = models.CharField(max_length=64, blank=True)
    county = models.CharField(max_length=64, blank=True)
    officer_id = models.CharField(max_length=128, blank=True)
    motion_class = models.CharField(max_length=32, blank=True)
    network_class = models.CharField(max_length=16, blank=True)
    battery_pct = models.FloatField(null=True, blank=True)
    attestation_status = models.CharField(max_length=32, blank=True)
    prev_hash = models.CharField(max_length=64, blank=True)
    chain_seq = models.BigIntegerField(default=0)

    class Meta:
        managed = False
        db_table = "eel_telemetry_signals"
        verbose_name = "Telemetry Signal"
        verbose_name_plural = "Telemetry Signals"
        ordering = ["-occurred_at"]

    def __str__(self) -> str:
        return f"{self.signal_class} @ {self.occurred_at} [{self.device_id}]"


class TelemetryAlert(models.Model):
    """Anomaly / alert record from eel_telemetry_alerts."""

    class Severity(models.TextChoices):
        CRITICAL = "critical", "Critical"
        HIGH = "high", "High"
        MEDIUM = "medium", "Medium"
        LOW = "low", "Low"

    alert_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    signal_class = models.CharField(max_length=32)
    severity = models.CharField(max_length=16, choices=Severity.choices)
    description = models.TextField()
    detected_at = models.DateTimeField(db_index=True)
    resolved_at = models.DateTimeField(null=True, blank=True)
    resolver_svid = models.CharField(max_length=256, blank=True)
    station_id = models.CharField(max_length=64, blank=True)
    county = models.CharField(max_length=64, blank=True)
    officer_id = models.CharField(max_length=128, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_alerts"
        verbose_name = "Telemetry Alert"
        verbose_name_plural = "Telemetry Alerts"
        ordering = ["-detected_at"]

    def __str__(self) -> str:
        return f"{self.severity.upper()} {self.signal_class} @ {self.detected_at}"


class TelemetryDeviceSnapshot(models.Model):
    """Daily device state snapshot from eel_telemetry_device_snapshots."""

    snapshot_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    snapshot_date = models.DateField(db_index=True)
    last_battery_pct = models.FloatField(null=True, blank=True)
    last_geo_lat = models.FloatField(null=True, blank=True)
    last_geo_lon = models.FloatField(null=True, blank=True)
    last_attestation_status = models.CharField(max_length=32, blank=True)
    signal_count = models.BigIntegerField(default=0)
    drop_count = models.IntegerField(default=0)
    session_count = models.IntegerField(default=0)
    merkle_root = models.CharField(max_length=64, blank=True)
    tlog_anchor = models.CharField(max_length=256, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_device_snapshots"
        verbose_name = "Device Snapshot"
        verbose_name_plural = "Device Snapshots"
        ordering = ["-snapshot_date"]

    def __str__(self) -> str:
        return f"Snapshot {self.device_id} @ {self.snapshot_date}"


class TelemetryOfficerSession(models.Model):
    """Officer login/logout session from eel_telemetry_officer_sessions."""

    session_id = models.UUIDField(primary_key=True)
    officer_id = models.CharField(max_length=128, db_index=True)
    device_id = models.UUIDField(db_index=True)
    login_at = models.DateTimeField()
    logout_at = models.DateTimeField(null=True, blank=True)
    station_id = models.CharField(max_length=64, blank=True)
    county = models.CharField(max_length=64, blank=True)
    geo_lat = models.FloatField(null=True, blank=True)
    geo_lon = models.FloatField(null=True, blank=True)
    biometric_verified = models.BooleanField(default=False)
    anomaly_flag = models.BooleanField(default=False)
    anomaly_description = models.TextField(blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_officer_sessions"
        verbose_name = "Officer Session"
        verbose_name_plural = "Officer Sessions"
        ordering = ["-login_at"]

    def __str__(self) -> str:
        return f"Session {self.officer_id} on {self.device_id} @ {self.login_at}"


class TelemetryMerkleSnapshot(models.Model):
    """Merkle tree snapshot record from eel_telemetry_merkle_snapshots."""

    snapshot_id = models.UUIDField(primary_key=True)
    device_id = models.UUIDField(db_index=True)
    period_start = models.DateTimeField()
    period_end = models.DateTimeField()
    leaf_count = models.IntegerField(default=0)
    merkle_root = models.CharField(max_length=64)
    tlog_entry_id = models.CharField(max_length=256, blank=True)
    anchored_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_merkle_snapshots"
        verbose_name = "Merkle Snapshot"
        verbose_name_plural = "Merkle Snapshots"
        ordering = ["-period_start"]

    def __str__(self) -> str:
        return f"Merkle {self.device_id} {self.period_start} -> {self.period_end}"


class TelemetryQueryAudit(models.Model):
    """Audit log of admin queries from eel_telemetry_queries."""

    query_id = models.UUIDField(primary_key=True)
    actor_svid = models.CharField(max_length=256, db_index=True)
    query_dsl = models.JSONField()
    executed_at = models.DateTimeField(db_index=True)
    row_count = models.BigIntegerField(default=0)
    result_sha256 = models.CharField(max_length=64, blank=True)
    evidence_exported = models.BooleanField(default=False)
    preset_name = models.CharField(max_length=64, blank=True)
    nl_text = models.TextField(blank=True)

    class Meta:
        managed = False
        db_table = "eel_telemetry_queries"
        verbose_name = "Query Audit"
        verbose_name_plural = "Query Audit Log"
        ordering = ["-executed_at"]

    def __str__(self) -> str:
        return f"Query {self.query_id} by {self.actor_svid} @ {self.executed_at}"
