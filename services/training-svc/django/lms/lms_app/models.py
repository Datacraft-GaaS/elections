from django.db import models
from uuid6 import uuid7


def uuid7str() -> str:
    return str(uuid7())


class TargetAudience(models.TextChoices):
    PO = "PO", "Presiding Officer"
    DPO = "DPO", "Deputy Presiding Officer"
    CLERK = "CLERK", "Clerk"
    SUPPORT = "SUPPORT", "Support Staff"
    TRAINER = "TRAINER", "Trainer"


class ModuleType(models.TextChoices):
    CORE = "CORE", "Core"
    REFRESHER = "REFRESHER", "Refresher"
    BRIEFING = "BRIEFING", "Briefing"
    TRAINER_CERT = "TRAINER_CERT", "Trainer Certification"


class EnrollmentStatus(models.TextChoices):
    NOT_STARTED = "NOT_STARTED", "Not Started"
    IN_PROGRESS = "IN_PROGRESS", "In Progress"
    PASSED = "PASSED", "Passed"
    FAILED = "FAILED", "Failed"
    RETAKE_REQUIRED = "RETAKE_REQUIRED", "Retake Required"


class ProgressScope(models.TextChoices):
    NATIONAL = "NATIONAL", "National"
    COUNTY = "COUNTY", "County"
    CONSTITUENCY = "CONSTITUENCY", "Constituency"


class Lesson(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    course = models.ForeignKey(Course, on_delete=models.CASCADE, related_name="lessons")
    module = models.ForeignKey(CourseModule, on_delete=models.CASCADE, related_name="lessons")
    order_index = models.PositiveSmallIntegerField(default=0)
    title = models.CharField(max_length=500)
    content_markdown = models.TextField()
    duration_minutes = models.PositiveSmallIntegerField(default=15)
    class Meta:
        db_table = "training_lms_lessons"
        ordering = ["order_index"]
        unique_together = [["course", "order_index"]]

class LessonAttempt(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    worker_id = models.UUIDField()
    lesson = models.ForeignKey(Lesson, on_delete=models.CASCADE, related_name="attempts")
    attempted_at = models.DateTimeField(auto_now_add=True)
    completed = models.BooleanField(default=False)
    score_pct = models.DecimalField(max_digits=5, decimal_places=2, null=True, blank=True)
    idempotency_key = models.CharField(max_length=128, unique=True)
    class Meta:
        db_table = "training_lms_lesson_attempts"
        ordering = ["-attempted_at"]

class CertAuditLog(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    event_type = models.CharField(max_length=30)
    certificate_id = models.UUIDField(null=True, blank=True)
    worker_id = models.UUIDField()
    recorded_at = models.DateTimeField(auto_now_add=True)
    proof_hash = models.CharField(max_length=64)
    correlation_key = models.CharField(max_length=64)
    class Meta:
        db_table = "training_lms_cert_audit"
        ordering = ["-recorded_at"]

class CourseModule(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    title = models.CharField(max_length=500)
    target_audience = models.CharField(max_length=30, choices=TargetAudience.choices)
    language = models.CharField(max_length=20)
    content_markdown = models.TextField()
    asset_ids = models.JSONField(default=list)
    case_study_ids = models.JSONField(default=list)
    pass_mark_pct = models.PositiveSmallIntegerField(default=80)
    max_retakes = models.PositiveSmallIntegerField(default=3)
    module_type = models.CharField(max_length=30, choices=ModuleType.choices)
    version = models.PositiveIntegerField(default=1)
    published_at = models.DateTimeField(null=True, blank=True)
    expiry_date = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = True
        db_table = "training_lms_modules"
        ordering = ["title"]

    def __str__(self) -> str:
        return f"CourseModule({self.title[:60]}, {self.module_type})"


class WorkerEnrollment(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    worker_id = models.UUIDField()
    module = models.ForeignKey(
        CourseModule, on_delete=models.CASCADE, related_name="enrollments"
    )
    status = models.CharField(
        max_length=30,
        choices=EnrollmentStatus.choices,
        default=EnrollmentStatus.NOT_STARTED,
    )
    attempts = models.PositiveSmallIntegerField(default=0)
    best_score_pct = models.DecimalField(max_digits=5, decimal_places=2, default=0)
    completed_at = models.DateTimeField(null=True, blank=True)
    certified = models.BooleanField(default=False)

    class Meta:
        managed = True
        db_table = "training_lms_enrollments"
        unique_together = [["worker_id", "module"]]

    def __str__(self) -> str:
        return f"WorkerEnrollment({self.worker_id}, {self.module_id}, {self.status})"


class WorkerCertificate(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    worker_id = models.UUIDField()
    station_code = models.CharField(max_length=20)
    modules_completed = models.JSONField(default=list)
    issued_at = models.DateTimeField(auto_now_add=True)
    expires_at = models.DateTimeField()
    revoked = models.BooleanField(default=False)
    revoked_at = models.DateTimeField(null=True, blank=True)
    revoked_by = models.UUIDField(null=True, blank=True)
    certificate_hash = models.CharField(max_length=64, blank=True)
    qr_verification_url = models.URLField()

    class Meta:
        managed = True
        db_table = "training_lms_certificates"
        ordering = ["-issued_at"]

    def __str__(self) -> str:
        return f"WorkerCertificate({self.worker_id}, {self.station_code})"


class TrainingProgress(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    scope = models.CharField(max_length=20, choices=ProgressScope.choices)
    scope_code = models.CharField(max_length=20)
    snapshot_at = models.DateTimeField(auto_now_add=True)
    total_workers = models.PositiveIntegerField(default=0)
    certified = models.PositiveIntegerField(default=0)
    refresher_pending = models.PositiveIntegerField(default=0)
    not_certified = models.PositiveIntegerField(default=0)
    certification_rate_pct = models.DecimalField(
        max_digits=5, decimal_places=2, default=0
    )

    class Meta:
        managed = True
        db_table = "training_lms_progress"
        ordering = ["-snapshot_at"]

    def __str__(self) -> str:
        return f"TrainingProgress({self.scope}, {self.scope_code}, {self.snapshot_at})"
