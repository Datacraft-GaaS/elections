from django.contrib import admin
from django.utils import timezone

from .models import (
    ContentItem,
    ContentStatus,
    ContentVersion,
    DistributionEvent,
    FactCheckReview,
)


@admin.register(ContentItem)
class ContentItemAdmin(admin.ModelAdmin):
    list_display = ["title", "language", "format", "status", "expiry_date"]
    list_filter = ["status", "language", "format", "topic_category"]
    search_fields = ["title", "election_cycle_tag"]
    readonly_fields = ["id", "created_at", "updated_at"]
    actions = ["submit_for_review", "approve", "publish"]

    @admin.action(description="Submit selected items for fact-check review")
    def submit_for_review(self, request, queryset):
        updated = queryset.filter(status=ContentStatus.DRAFT).update(
            status=ContentStatus.IN_FACT_CHECK
        )
        self.message_user(request, f"{updated} item(s) submitted for review.")

    @admin.action(description="Approve selected items")
    def approve(self, request, queryset):
        updated = queryset.filter(
            status__in=[ContentStatus.FACT_CHECK_PASSED, ContentStatus.IN_APPROVAL]
        ).update(
            status=ContentStatus.APPROVED,
            approved_by=None,
            approved_at=timezone.now(),
        )
        self.message_user(request, f"{updated} item(s) approved.")

    @admin.action(description="Publish selected items")
    def publish(self, request, queryset):
        updated = queryset.filter(status=ContentStatus.APPROVED).update(
            status=ContentStatus.PUBLISHED
        )
        self.message_user(request, f"{updated} item(s) published.")


@admin.register(ContentVersion)
class ContentVersionAdmin(admin.ModelAdmin):
    list_display = ["id", "content_item", "version_number", "created_by", "created_at"]
    list_filter = ["created_at"]
    readonly_fields = ["id", "created_at"]


@admin.register(FactCheckReview)
class FactCheckReviewAdmin(admin.ModelAdmin):
    list_display = ["id", "content_item", "reviewer_id", "decision", "completed_at"]
    list_filter = ["decision"]
    readonly_fields = ["id"]


@admin.register(DistributionEvent)
class DistributionEventAdmin(admin.ModelAdmin):
    list_display = [
        "id",
        "content_item",
        "channel",
        "county_code",
        "dispatched_at",
        "quantity",
    ]
    list_filter = ["channel"]
    readonly_fields = ["id", "dispatched_at"]
