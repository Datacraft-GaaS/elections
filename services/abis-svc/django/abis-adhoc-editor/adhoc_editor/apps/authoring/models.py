"""Django models mirroring V029 tables (managed=False -- Flyway owns the schema)."""

import uuid

from django.db import models


class AdhocReport(models.Model):
    class DataClass(models.TextChoices):
        RESTRICTED = "restricted", "Restricted"
        SENSITIVE = "sensitive", "Sensitive"
        PUBLIC = "public", "Public"

    class Visibility(models.TextChoices):
        PRIVATE = "private", "Private"
        SHARED = "shared", "Shared"
        TEAM = "team", "Team"
        ORG = "org", "Organisation"

    adhoc_id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    slug = models.SlugField(max_length=200)
    title = models.CharField(max_length=500)
    description = models.TextField(blank=True, null=True)
    author_id = models.UUIDField(db_index=True)
    data_class = models.CharField(
        max_length=20,
        choices=DataClass.choices,
        default=DataClass.SENSITIVE,
    )
    visibility = models.CharField(
        max_length=20,
        choices=Visibility.choices,
        default=Visibility.PRIVATE,
    )
    published_as_definition_id = models.UUIDField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    archived_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_adhoc_reports"
        ordering = ["-updated_at"]
        indexes = [
            models.Index(fields=["visibility", "-updated_at"]),
        ]

    def __str__(self) -> str:
        return f"{self.title} ({self.slug})"


class AdhocReportVersion(models.Model):
    class SourceType(models.TextChoices):
        SQL = "sql", "SQL"
        QUERY_BUILDER = "query_builder", "Query Builder"
        MARKDOWN = "markdown", "Markdown"

    version_id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    adhoc = models.ForeignKey(
        AdhocReport,
        on_delete=models.CASCADE,
        db_column="adhoc_id",
        related_name="versions",
    )
    version_number = models.IntegerField()
    source_type = models.CharField(
        max_length=20, choices=SourceType.choices, default=SourceType.SQL
    )
    source = models.TextField()
    parameters = models.JSONField(default=list)
    allowed_tables = models.JSONField(default=list)
    column_projection = models.JSONField(null=True, blank=True)
    render_pipeline = models.JSONField(default=list)
    created_at = models.DateTimeField(auto_now_add=True)
    created_by = models.UUIDField()
    commit_message = models.TextField(blank=True, null=True)
    linted_at = models.DateTimeField(null=True, blank=True)
    lint_report = models.JSONField(default=dict)
    safety_score = models.FloatField(default=0.0)

    class Meta:
        managed = False
        db_table = "eel_adhoc_report_versions"
        ordering = ["-created_at"]
        unique_together = [("adhoc", "version_number")]

    def __str__(self) -> str:
        return f"v{self.version_number} of {self.adhoc_id}"


class AdhocPreview(models.Model):
    class PreviewFormat(models.TextChoices):
        JSON = "json", "JSON"
        MARKDOWN = "markdown", "Markdown"
        DOCX = "docx", "DOCX"
        CSV = "csv", "CSV"

    preview_id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    version = models.ForeignKey(
        AdhocReportVersion,
        on_delete=models.CASCADE,
        db_column="version_id",
        related_name="previews",
    )
    parameter_values = models.JSONField(default=dict)
    sample_limit = models.IntegerField(default=100)
    rows_returned = models.IntegerField(null=True, blank=True)
    elapsed_ms = models.IntegerField(null=True, blank=True)
    preview_format = models.CharField(
        max_length=20, choices=PreviewFormat.choices, default=PreviewFormat.JSON
    )
    preview_bytes = models.BinaryField(null=True, blank=True)
    requested_by = models.UUIDField()
    requested_at = models.DateTimeField(auto_now_add=True)
    expires_at = models.DateTimeField()
    error_message = models.TextField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_adhoc_previews"
        ordering = ["-requested_at"]

    def __str__(self) -> str:
        return f"Preview {self.preview_id} ({self.preview_format})"


class AdhocShare(models.Model):
    class Permission(models.TextChoices):
        VIEW = "view", "View"
        RUN = "run", "Run"
        EDIT = "edit", "Edit"
        ADMIN = "admin", "Admin"

    share_id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    adhoc = models.ForeignKey(
        AdhocReport,
        on_delete=models.CASCADE,
        db_column="adhoc_id",
        related_name="shares",
    )
    shared_with = models.UUIDField(db_index=True)
    permission = models.CharField(max_length=20, choices=Permission.choices)
    granted_by = models.UUIDField()
    granted_at = models.DateTimeField(auto_now_add=True)
    revoked_at = models.DateTimeField(null=True, blank=True)
    expires_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "eel_adhoc_shares"
        ordering = ["-granted_at"]

    def __str__(self) -> str:
        return f"Share {self.share_id} ({self.permission})"


class AdhocSnippet(models.Model):
    class Kind(models.TextChoices):
        CTE = "cte", "CTE"
        WHERE = "where", "WHERE clause"
        SELECT = "select", "SELECT clause"
        MARKDOWN_PARTIAL = "markdown_partial", "Markdown partial"

    class Visibility(models.TextChoices):
        PRIVATE = "private", "Private"
        SHARED = "shared", "Shared"
        ORG = "org", "Organisation"

    snippet_id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    author_id = models.UUIDField(db_index=True)
    name = models.CharField(max_length=200)
    body = models.TextField()
    kind = models.CharField(max_length=30, choices=Kind.choices)
    visibility = models.CharField(
        max_length=20, choices=Visibility.choices, default=Visibility.PRIVATE
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        managed = False
        db_table = "eel_adhoc_snippets"
        ordering = ["name"]
        unique_together = [("author_id", "name")]

    def __str__(self) -> str:
        return f"{self.name} ({self.kind})"
