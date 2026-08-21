"""Django admin configuration for adjudication case management."""

from django.contrib import admin, messages
from django.utils.html import format_html

from .models import AdjudicationCase, PanelVote, ReviewerAction


@admin.register(AdjudicationCase)
class AdjudicationCaseAdmin(admin.ModelAdmin):
    list_display = [
        "id",
        "priority",
        "state",
        "tier",
        "assigned_to",
        "opened_at",
        "sla_hours_remaining_display",
        "sla_breach_flag",
    ]
    list_filter = ["state", "priority", "tier"]
    search_fields = ["id", "evidence_bundle_ref", "assigned_to"]
    readonly_fields = ["id", "opened_at", "created_at", "updated_at"]
    ordering = ["-priority", "sla_deadline_at"]

    def get_readonly_fields(self, request, obj=None):
        """Lock decision fields once a case is decided or withdrawn."""
        ro = list(super().get_readonly_fields(request, obj))
        if obj and obj.is_decided():
            ro += [
                "decision_type",
                "decision_subject_id",
                "decision_reason",
                "decided_at",
                "state",
            ]
        return ro

    @admin.display(description="SLA remaining (h)")
    def sla_hours_remaining_display(self, obj: AdjudicationCase) -> str:
        hours = obj.sla_hours_remaining()
        if hours < 0:
            return format_html('<span style="color:red">{:.1f}h</span>', hours)
        if hours < 2:
            return format_html('<span style="color:orange">{:.1f}h</span>', hours)
        return f"{hours:.1f}h"

    @admin.display(description="SLA")
    def sla_breach_flag(self, obj: AdjudicationCase) -> str:
        if obj.is_sla_breached():
            return format_html('<span style="color:red">BREACHED</span>')
        return format_html('<span style="color:green">OK</span>')

    @admin.action(description="Assign selected cases to me")
    def assign_to_me(self, request, queryset):
        updated = queryset.filter(state__in=["Open", "Escalated", "Reopened"]).update(
            assigned_to=request.user.pk,
            state="Assigned",
        )
        self.message_user(
            request, f"{updated} case(s) assigned to you.", messages.SUCCESS
        )

    @admin.action(description="Escalate selected cases to Tier 2")
    def escalate_to_tier2(self, request, queryset):
        updated = queryset.filter(state="InReview").update(
            state="Escalated",
            tier="Tier2Supervisor",
        )
        self.message_user(
            request, f"{updated} case(s) escalated to Tier 2.", messages.SUCCESS
        )

    @admin.action(description="Request Tier 3 panel for selected cases")
    def request_tier3_panel(self, request, queryset):
        updated = queryset.filter(state="Escalated").update(
            state="Escalated",
            tier="Tier3Panel",
        )
        self.message_user(
            request, f"{updated} case(s) queued for Tier 3 panel.", messages.SUCCESS
        )

    @admin.action(description="Add evidence (redirect to upload)")
    def add_evidence(self, request, queryset):
        # In a full implementation, redirect to the evidence upload view.
        # For now, show a message directing the reviewer to the case detail page.
        self.message_user(
            request,
            "Use the case detail page to add evidence via the HTMX evidence upload form.",
            messages.INFO,
        )

    actions = [assign_to_me, escalate_to_tier2, request_tier3_panel, add_evidence]


@admin.register(ReviewerAction)
class ReviewerActionAdmin(admin.ModelAdmin):
    list_display = ["id", "case", "reviewer_id", "action_type", "taken_at"]
    list_filter = ["action_type"]
    search_fields = ["case__id", "reviewer_id"]
    readonly_fields = [
        "id",
        "case",
        "reviewer_id",
        "action_type",
        "comment",
        "evidence_added",
        "taken_at",
        "hsm_signature",
        "public_key_hex",
        "prev_action_hash",
        "canonical_json",
    ]

    def has_add_permission(self, request):
        return False

    def has_change_permission(self, request, obj=None):
        return False

    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(PanelVote)
class PanelVoteAdmin(admin.ModelAdmin):
    list_display = ["id", "case", "reviewer_id", "vote", "voted_at"]
    list_filter = ["vote"]
    search_fields = ["case__id", "reviewer_id"]
    readonly_fields = [
        "id",
        "case",
        "reviewer_id",
        "vote",
        "rationale",
        "voted_at",
        "hsm_signature",
        "public_key_hex",
    ]

    def has_add_permission(self, request):
        return False

    def has_change_permission(self, request, obj=None):
        return False

    def has_delete_permission(self, request, obj=None):
        return False
