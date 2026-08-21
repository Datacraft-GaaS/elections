"""Django admin registrations for ABIS reports."""

import hashlib
import uuid
from typing import Any

from django.contrib import admin, messages
from django.db import models as db_models
from django.http import (
    HttpRequest,
    HttpResponse,
    HttpResponseBase,
    StreamingHttpResponse,
)
from django.utils import timezone

from .models import (
    ReportAccessLog,
    ReportArtifact,
    ReportDefinition,
    ReportDelivery,
    ReportRun,
    ReportSubscriber,
    ReportSubscription,
    ReportTlogAnchor,
)


# ---------------------------------------------------------------------------
# Report Definition
# ---------------------------------------------------------------------------


@admin.register(ReportDefinition)
class ReportDefinitionAdmin(admin.ModelAdmin):
    list_display = [
        "slug",
        "report_class",
        "audience",
        "default_schedule",
        "retention_years",
        "retired_at",
    ]
    list_filter = ["report_class", "data_class", "requires_hsm_sign"]
    search_fields = ["slug", "title"]
    readonly_fields = [
        "definition_id",
        "created_at",
        "updated_at",
        "cosign_signature",
    ]
    actions = ["duplicate_as_ad_hoc", "run_now"]

    @admin.action(description="Duplicate as ad-hoc definition")
    def duplicate_as_ad_hoc(
        self, request: HttpRequest, queryset: db_models.QuerySet
    ) -> None:
        for definition in queryset:
            new_id = uuid.uuid4()
            new_slug = f"{definition.slug}-adhoc-{new_id.hex[:6]}"
            ReportDefinition.objects.create(
                definition_id=new_id,
                slug=new_slug,
                title=f"[Ad hoc] {definition.title}",
                report_class="ad_hoc",
                audience=definition.audience,
                data_class=definition.data_class,
                sql_template_ref=definition.sql_template_ref,
                renderer_pipeline=definition.renderer_pipeline,
                requires_hsm_sign=False,
                requires_tlog_anchor=False,
                default_schedule=None,
                retention_years=1,
                template_version=definition.template_version,
                created_at=timezone.now(),
                updated_at=timezone.now(),
            )
            self.message_user(
                request,
                f"Created ad-hoc copy: {new_slug}",
                messages.SUCCESS,
            )

    @admin.action(description="Queue a run now (default window: today)")
    def run_now(self, request: HttpRequest, queryset: db_models.QuerySet) -> None:
        now = timezone.now()
        window_from = now.replace(hour=0, minute=0, second=0, microsecond=0)
        window_to = now
        for definition in queryset:
            run = ReportRun(
                run_id=uuid.uuid4(),
                definition=definition,
                slug=definition.slug,
                window_from=window_from,
                window_to=window_to,
                status="queued",
                requested_by=str(request.user),
                requested_at=now,
            )
            run.save()
            self.message_user(
                request,
                f"Queued run {run.run_id} for {definition.slug}",
                messages.SUCCESS,
            )


# ---------------------------------------------------------------------------
# Report Run
# ---------------------------------------------------------------------------


@admin.register(ReportRun)
class ReportRunAdmin(admin.ModelAdmin):
    list_display = [
        "run_id",
        "slug",
        "status",
        "window_from",
        "window_to",
        "rows_extracted",
        "total_ms",
    ]
    list_filter = ["status", "definition__report_class"]
    search_fields = ["run_id", "slug"]
    actions = ["cancel_run", "retry_run"]

    def get_readonly_fields(self, request: HttpRequest, obj: Any = None) -> list[str]:
        if obj and obj.status in ("complete", "failed", "cancelled"):
            return [f.name for f in obj._meta.fields]
        return ["run_id", "requested_at"]

    @admin.action(description="Cancel selected runs")
    def cancel_run(self, request: HttpRequest, queryset: db_models.QuerySet) -> None:
        cancellable = queryset.filter(status__in=["queued", "extracting", "rendering"])
        count = cancellable.update(status="cancelled")
        self.message_user(request, f"Cancelled {count} run(s).", messages.SUCCESS)

    @admin.action(description="Retry failed runs (re-queue)")
    def retry_run(self, request: HttpRequest, queryset: db_models.QuerySet) -> None:
        failed = queryset.filter(status="failed")
        count = failed.update(status="queued", failure_reason=None, started_at=None)
        self.message_user(request, f"Re-queued {count} run(s).", messages.SUCCESS)


# ---------------------------------------------------------------------------
# Report Artifact
# ---------------------------------------------------------------------------


def _stream_artifact(blob_ref: str, content_type: str) -> StreamingHttpResponse:
    """Stream artifact bytes from MinIO using django-storages."""
    from django.core.files.storage import default_storage

    def _iter() -> Any:
        with default_storage.open(blob_ref, "rb") as f:
            while chunk := f.read(65536):
                yield chunk

    resp = StreamingHttpResponse(_iter(), content_type=content_type)
    resp["Content-Disposition"] = f'attachment; filename="{blob_ref.split("/")[-1]}"'
    return resp


_FORMAT_CONTENT_TYPES = {
    "pdf": "application/pdf",
    "csv": "text/csv",
    "json": "application/json",
    "xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "html": "text/html",
    "png": "image/png",
    "svg": "image/svg+xml",
    "parquet": "application/octet-stream",
}


@admin.register(ReportArtifact)
class ReportArtifactAdmin(admin.ModelAdmin):
    list_display = [
        "artifact_id",
        "format",
        "size_human_display",
        "sha256_truncated",
        "page_count",
        "row_count",
        "rendered_at",
    ]
    list_filter = ["format"]
    search_fields = ["artifact_id", "run__slug"]
    readonly_fields = [f.name for f in ReportArtifact._meta.fields]
    actions = ["download_artifact"]

    @admin.display(description="Size")
    def size_human_display(self, obj: ReportArtifact) -> str:
        return obj.size_human

    @admin.display(description="SHA-256 (truncated)")
    def sha256_truncated(self, obj: ReportArtifact) -> str:
        return obj.sha256_hex[:16] + "..."

    @admin.action(description="Download artifact (streams via console, logs access)")
    def download_artifact(
        self, request: HttpRequest, queryset: db_models.QuerySet
    ) -> HttpResponseBase:
        if queryset.count() != 1:
            self.message_user(
                request, "Select exactly one artifact to download.", messages.WARNING
            )
            return HttpResponse(status=400)

        first = queryset.first()
        if first is None:
            return HttpResponse(status=400)
        artifact: ReportArtifact = first
        content_type = _FORMAT_CONTENT_TYPES.get(
            artifact.format, "application/octet-stream"
        )

        ua_raw = request.META.get("HTTP_USER_AGENT", "")
        ua_hash = hashlib.sha256(ua_raw.encode()).digest()

        ip = request.META.get("REMOTE_ADDR", "")
        ip_prefix = ".".join(ip.split(".")[:3]) + ".0/24" if "." in ip else ip[:12]

        ReportAccessLog.objects.create(
            access_id=uuid.uuid4(),
            artifact=artifact,
            subject_id=str(request.user) if request.user.is_authenticated else None,
            ip_prefix=ip_prefix,
            user_agent_hash=ua_hash,
            accessed_at=timezone.now(),
        )

        return _stream_artifact(artifact.blob_ref, content_type)


# ---------------------------------------------------------------------------
# Tlog Anchor
# ---------------------------------------------------------------------------


@admin.register(ReportTlogAnchor)
class ReportTlogAnchorAdmin(admin.ModelAdmin):
    list_display = ["anchor_id", "tlog_leaf_index", "anchored_at", "run"]
    search_fields = ["anchor_id", "run__slug"]
    readonly_fields = [f.name for f in ReportTlogAnchor._meta.fields]


# ---------------------------------------------------------------------------
# Subscriber
# ---------------------------------------------------------------------------


@admin.register(ReportSubscriber)
class ReportSubscriberAdmin(admin.ModelAdmin):
    list_display = [
        "name",
        "channel",
        "audience",
        "preferred_format",
        "verified_at",
        "suspended_at",
    ]
    list_filter = ["channel", "audience"]
    search_fields = ["name"]
    readonly_fields = ["subscriber_id", "endpoint_hash", "created_at"]


# ---------------------------------------------------------------------------
# Subscription
# ---------------------------------------------------------------------------


@admin.register(ReportSubscription)
class ReportSubscriptionAdmin(admin.ModelAdmin):
    list_display = [
        "subscription_id",
        "subscriber",
        "definition",
        "format",
        "created_at",
        "cancelled_at",
    ]
    list_filter = ["format"]
    search_fields = ["subscriber__name", "definition__slug"]
    readonly_fields = ["subscription_id", "created_at"]


# ---------------------------------------------------------------------------
# Delivery
# ---------------------------------------------------------------------------


@admin.register(ReportDelivery)
class ReportDeliveryAdmin(admin.ModelAdmin):
    list_display = [
        "delivery_id",
        "run",
        "status",
        "attempt_count",
        "last_attempt_at",
        "delivered_at",
    ]
    list_filter = ["status"]
    search_fields = ["delivery_id", "run__slug"]
    readonly_fields = ["delivery_id"]
    actions = ["retry_delivery", "cancel_delivery"]

    @admin.action(description="Retry selected deliveries")
    def retry_delivery(
        self, request: HttpRequest, queryset: db_models.QuerySet
    ) -> None:
        count = queryset.filter(status__in=["failed", "abandoned"]).update(
            status="pending",
            next_attempt_at=timezone.now(),
        )
        self.message_user(request, f"Re-queued {count} delivery(s).", messages.SUCCESS)

    @admin.action(description="Cancel selected deliveries")
    def cancel_delivery(
        self, request: HttpRequest, queryset: db_models.QuerySet
    ) -> None:
        count = queryset.filter(status__in=["pending", "failed"]).update(
            status="abandoned"
        )
        self.message_user(request, f"Cancelled {count} delivery(s).", messages.SUCCESS)


# ---------------------------------------------------------------------------
# Access Log
# ---------------------------------------------------------------------------


@admin.register(ReportAccessLog)
class ReportAccessLogAdmin(admin.ModelAdmin):
    list_display = ["access_id", "artifact", "subject_id", "ip_prefix", "accessed_at"]
    list_filter = []
    search_fields = ["subject_id", "artifact__run__slug"]
    readonly_fields = [f.name for f in ReportAccessLog._meta.fields]

    def has_add_permission(self, request: HttpRequest) -> bool:
        return False

    def has_change_permission(self, request: HttpRequest, obj: Any = None) -> bool:
        return False
