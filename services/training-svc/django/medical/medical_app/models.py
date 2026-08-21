from uuid6 import uuid7
from django.db import models


def uuid7str() -> str:
    return str(uuid7())


class MedicalIncident(models.Model):
    """Election-day medical incident.

    IMPORTANT: No PHI fields. Records aggregate counts only,
    never patient names, IDs, diagnoses, DOB, or health records.
    """

    INCIDENT_TYPE_CHOICES = [
        ("injury", "Injury"),
        ("illness", "Illness"),
        ("cardiac", "Cardiac"),
        ("other", "Other"),
    ]
    SEVERITY_CHOICES = [
        ("minor", "Minor"),
        ("moderate", "Moderate"),
        ("critical", "Critical"),
    ]
    STATUS_CHOICES = [
        ("reported", "Reported"),
        ("dispatched", "Dispatched"),
        ("resolved", "Resolved"),
        ("escalated", "Escalated"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    station_id = models.CharField(max_length=30)
    incident_type = models.CharField(max_length=30, choices=INCIDENT_TYPE_CHOICES)
    severity = models.CharField(max_length=20, choices=SEVERITY_CHOICES)
    reported_at = models.DateTimeField(auto_now_add=True)
    reporter_id = models.UUIDField()
    status = models.CharField(max_length=30, choices=STATUS_CHOICES, default="reported")
    persons_affected = models.PositiveIntegerField(default=1)

    class Meta:
        db_table = "training_medical_incidents"
        managed = True

    def __str__(self) -> str:
        return f"Incident({self.incident_type}, {self.station_id}, {self.status})"


class DispatchRequest(models.Model):
    DISPATCHED_TO_CHOICES = [
        ("KRCS", "Kenya Red Cross Society"),
        ("MoH", "Ministry of Health"),
        ("NDMU", "National Disaster Management Unit"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    incident = models.ForeignKey(
        MedicalIncident,
        on_delete=models.CASCADE,
        related_name="dispatches",
    )
    dispatched_to = models.CharField(max_length=20, choices=DISPATCHED_TO_CHOICES)
    dispatched_at = models.DateTimeField(auto_now_add=True)
    eta_minutes = models.PositiveSmallIntegerField(null=True, blank=True)
    acknowledged_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "training_medical_dispatch_requests"
        managed = True

    def __str__(self) -> str:
        return f"Dispatch({self.dispatched_to}, incident={self.incident_id})"


class ClusterAlert(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    county_code = models.CharField(max_length=10)
    incident_count = models.PositiveIntegerField(default=0)
    alert_threshold = models.PositiveIntegerField(default=3)
    raised_at = models.DateTimeField(auto_now_add=True)
    resolved_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "training_medical_cluster_alerts"
        managed = True

    def __str__(self) -> str:
        return f"ClusterAlert({self.county_code}, count={self.incident_count})"


class MedicalUnit(models.Model):
    UNIT_TYPE_CHOICES = [
        ("first_aid_team", "First Aid Team"),
        ("ambulance", "Ambulance"),
        ("field_hospital", "Field Hospital"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid7, editable=False)
    unit_id = models.CharField(max_length=50, unique=True)
    unit_type = models.CharField(max_length=30, choices=UNIT_TYPE_CHOICES)
    county = models.CharField(max_length=30)
    sub_county = models.CharField(max_length=50)
    gps_lat = models.DecimalField(max_digits=9, decimal_places=6, null=True, blank=True)
    gps_lon = models.DecimalField(max_digits=9, decimal_places=6, null=True, blank=True)
    contact_number = models.CharField(max_length=20)
    capacity = models.PositiveSmallIntegerField(default=1)
    locked = models.BooleanField(default=False)

    class Meta:
        db_table = "training_medical_units"
        managed = True

    def __str__(self) -> str:
        return f"MedicalUnit({self.unit_id}, {self.unit_type})"
