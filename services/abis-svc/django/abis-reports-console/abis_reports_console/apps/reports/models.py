"""Django models mirroring the V027 eel_report_* tables.

All models use managed = False because Rust migrations own the schema.
"""

from django.db import models


class ReportDefinition(models.Model):
    definition_id = models.UUIDField(primary_key=True)
    slug = models.TextField(unique=True)
    title = models.TextField()
    report_class = models.TextField()
    audience = models.JSONField()
    data_class = models.TextField()
    sql_template_ref = models.TextField(null=True, blank=True)
    renderer_pipeline = models.JSONField()
    requires_hsm_sign = models.BooleanField(default=False)
    requires_tlog_anchor = models.BooleanField(default=False)
    default_schedule = models.TextField(null=True, blank=True)
    timezone = models.TextField(default="Africa/Nairobi")
    retention_years = models.SmallIntegerField(default=7)
    template_version = models.TextField()
    cosign_signature = models.BinaryField(null=True, blank=True)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()
    retired_at = models.DateTimeField(null=True, blank=True)
    retirement_reason = models.TextField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_definitions"
        managed = False
        ordering = ["slug"]
        verbose_name = "Report Definition"
        verbose_name_plural = "Report Definitions"

    def __str__(self) -> str:
        return self.slug


class ReportRun(models.Model):
    run_id = models.UUIDField(primary_key=True)
    definition = models.ForeignKey(
        ReportDefinition,
        on_delete=models.PROTECT,
        db_column="definition_id",
    )
    slug = models.TextField()
    window_from = models.DateTimeField()
    window_to = models.DateTimeField()
    status = models.TextField(default="queued")
    requested_by = models.TextField()
    requested_at = models.DateTimeField()
    started_at = models.DateTimeField(null=True, blank=True)
    completed_at = models.DateTimeField(null=True, blank=True)
    rows_extracted = models.BigIntegerField(null=True, blank=True)
    extraction_ms = models.IntegerField(null=True, blank=True)
    render_ms = models.IntegerField(null=True, blank=True)
    total_ms = models.IntegerField(null=True, blank=True)
    failure_reason = models.TextField(null=True, blank=True)
    correlation_id = models.UUIDField(null=True, blank=True)
    trace_id = models.BinaryField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_runs"
        managed = False
        ordering = ["-requested_at"]
        verbose_name = "Report Run"
        verbose_name_plural = "Report Runs"

    def __str__(self) -> str:
        return f"{self.slug} / {self.run_id}"


class ReportArtifact(models.Model):
    artifact_id = models.UUIDField(primary_key=True)
    run = models.ForeignKey(
        ReportRun,
        on_delete=models.PROTECT,
        db_column="run_id",
        related_name="artifacts",
    )
    format = models.TextField()
    blob_ref = models.TextField()
    sha256 = models.BinaryField()
    size_bytes = models.BigIntegerField()
    page_count = models.IntegerField(null=True, blank=True)
    row_count = models.BigIntegerField(null=True, blank=True)
    hsm_signature = models.BinaryField(null=True, blank=True)
    hsm_key_role = models.TextField(null=True, blank=True)
    rendered_at = models.DateTimeField()

    class Meta:
        db_table = "eel_report_artifacts"
        managed = False
        ordering = ["format"]
        verbose_name = "Report Artifact"
        verbose_name_plural = "Report Artifacts"

    def __str__(self) -> str:
        return f"{self.artifact_id} / {self.format}"

    @property
    def sha256_hex(self) -> str:
        return bytes(self.sha256).hex() if self.sha256 else ""

    @property
    def size_human(self) -> str:
        size = self.size_bytes
        for unit in ("B", "KB", "MB", "GB"):
            if size < 1024:
                return f"{size:.1f} {unit}"
            size /= 1024
        return f"{size:.1f} TB"


class ReportTlogAnchor(models.Model):
    anchor_id = models.UUIDField(primary_key=True)
    run = models.ForeignKey(
        ReportRun,
        on_delete=models.PROTECT,
        db_column="run_id",
    )
    artifact = models.ForeignKey(
        ReportArtifact,
        on_delete=models.PROTECT,
        db_column="artifact_id",
    )
    tlog_leaf_index = models.BigIntegerField()
    tlog_root_hash = models.BinaryField()
    inclusion_proof = models.BinaryField()
    sth_size = models.BigIntegerField()
    witness_cosigs = models.JSONField(default=list)
    anchored_at = models.DateTimeField()

    class Meta:
        db_table = "eel_report_tlog_anchors"
        managed = False
        verbose_name = "Tlog Anchor"
        verbose_name_plural = "Tlog Anchors"

    def __str__(self) -> str:
        return f"leaf#{self.tlog_leaf_index}"


class ReportSubscriber(models.Model):
    subscriber_id = models.UUIDField(primary_key=True)
    name = models.TextField()
    channel = models.TextField()
    endpoint = models.TextField()
    endpoint_hash = models.BinaryField()
    audience = models.TextField()
    auth_ref = models.TextField(null=True, blank=True)
    pgp_pubkey_ref = models.TextField(null=True, blank=True)
    preferred_format = models.TextField(default="pdf")
    created_at = models.DateTimeField()
    verified_at = models.DateTimeField(null=True, blank=True)
    suspended_at = models.DateTimeField(null=True, blank=True)
    suspension_reason = models.TextField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_subscribers"
        managed = False
        ordering = ["name"]
        verbose_name = "Report Subscriber"
        verbose_name_plural = "Report Subscribers"

    def __str__(self) -> str:
        return f"{self.name} ({self.channel})"


class ReportSubscription(models.Model):
    subscription_id = models.UUIDField(primary_key=True)
    subscriber = models.ForeignKey(
        ReportSubscriber,
        on_delete=models.PROTECT,
        db_column="subscriber_id",
    )
    definition = models.ForeignKey(
        ReportDefinition,
        on_delete=models.PROTECT,
        db_column="definition_id",
    )
    format = models.TextField()
    filter = models.JSONField(default=dict)
    created_at = models.DateTimeField()
    cancelled_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_subscriptions"
        managed = False
        verbose_name = "Report Subscription"
        verbose_name_plural = "Report Subscriptions"

    def __str__(self) -> str:
        return f"{self.subscriber} -> {self.definition}"


class ReportDelivery(models.Model):
    delivery_id = models.UUIDField(primary_key=True)
    run = models.ForeignKey(
        ReportRun,
        on_delete=models.PROTECT,
        db_column="run_id",
    )
    subscription = models.ForeignKey(
        ReportSubscription,
        on_delete=models.PROTECT,
        db_column="subscription_id",
    )
    artifact = models.ForeignKey(
        ReportArtifact,
        on_delete=models.PROTECT,
        db_column="artifact_id",
    )
    status = models.TextField(default="pending")
    attempt_count = models.SmallIntegerField(default=0)
    max_attempts = models.SmallIntegerField(default=5)
    next_attempt_at = models.DateTimeField(null=True, blank=True)
    last_attempt_at = models.DateTimeField(null=True, blank=True)
    delivered_at = models.DateTimeField(null=True, blank=True)
    last_error = models.TextField(null=True, blank=True)
    remote_receipt_id = models.TextField(null=True, blank=True)
    remote_receipt_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_deliveries"
        managed = False
        ordering = ["-last_attempt_at"]
        verbose_name = "Report Delivery"
        verbose_name_plural = "Report Deliveries"

    def __str__(self) -> str:
        return f"{self.delivery_id} / {self.status}"


class ReportAccessLog(models.Model):
    access_id = models.UUIDField(primary_key=True)
    artifact = models.ForeignKey(
        ReportArtifact,
        on_delete=models.PROTECT,
        db_column="artifact_id",
    )
    subject_id = models.TextField(null=True, blank=True)
    ip_prefix = models.TextField(null=True, blank=True)
    user_agent_hash = models.BinaryField(null=True, blank=True)
    accessed_at = models.DateTimeField()
    byte_range = models.TextField(null=True, blank=True)

    class Meta:
        db_table = "eel_report_access_log"
        managed = False
        ordering = ["-accessed_at"]
        verbose_name = "Access Log Entry"
        verbose_name_plural = "Access Log Entries"

    def __str__(self) -> str:
        return f"{self.access_id} @ {self.accessed_at}"
