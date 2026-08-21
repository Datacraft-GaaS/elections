"""Django models mirroring Rust adjudication schema.

All models use managed=False: Rust Flyway migrations (V012-V014) own the schema.
Django's ORM provides read/write access to rows written by the Rust service.
"""

from django.db import models


class AdjudicationCase(models.Model):
    """Mirrors eel_adjudication_cases_v2."""

    PRIORITY_CHOICES = [
        ("Standard", "Standard"),
        ("Urgent", "Urgent"),
        ("ElectionDayHold", "Election Day Hold"),
    ]
    TIER_CHOICES = [
        ("Tier1Analyst", "Tier 1 Analyst"),
        ("Tier2Supervisor", "Tier 2 Supervisor"),
        ("Tier3Panel", "Tier 3 Panel"),
    ]
    STATE_CHOICES = [
        ("Open", "Open"),
        ("Assigned", "Assigned"),
        ("InReview", "In Review"),
        ("Decided", "Decided"),
        ("Escalated", "Escalated"),
        ("Withdrawn", "Withdrawn"),
        ("Reopened", "Reopened"),
    ]

    id = models.UUIDField(primary_key=True)
    opened_at = models.DateTimeField()
    priority = models.CharField(
        max_length=20, choices=PRIORITY_CHOICES, default="Standard"
    )
    tier = models.CharField(max_length=20, choices=TIER_CHOICES, default="Tier1Analyst")
    state = models.CharField(max_length=20, choices=STATE_CHOICES, default="Open")
    enrolment_job_id = models.UUIDField(null=True, blank=True)
    probable_match_subject_ids = models.JSONField(default=list)
    evidence_bundle_ref = models.UUIDField()
    assigned_to = models.UUIDField(null=True, blank=True)
    assigned_at = models.DateTimeField(null=True, blank=True)
    decided_at = models.DateTimeField(null=True, blank=True)
    decision_type = models.CharField(max_length=20, null=True, blank=True)
    decision_subject_id = models.UUIDField(null=True, blank=True)
    decision_reason = models.TextField(null=True, blank=True)
    sla_deadline_at = models.DateTimeField()
    created_at = models.DateTimeField(auto_now_add=False)
    updated_at = models.DateTimeField(auto_now=False)

    class Meta:
        db_table = "eel_adjudication_cases_v2"
        managed = False
        ordering = ["-priority", "sla_deadline_at"]
        verbose_name = "Adjudication Case"
        verbose_name_plural = "Adjudication Cases"

    def __str__(self) -> str:
        return f"Case {self.id} [{self.state}] {self.priority}"

    def sla_hours_remaining(self) -> float:
        from django.utils import timezone

        delta = self.sla_deadline_at - timezone.now()
        return delta.total_seconds() / 3600

    def is_sla_breached(self) -> bool:
        return self.sla_hours_remaining() < 0

    def is_decided(self) -> bool:
        return self.state in ("Decided", "Withdrawn")


class ReviewerAction(models.Model):
    """Mirrors eel_reviewer_actions (append-only)."""

    ACTION_CHOICES = [
        ("Assign", "Assign"),
        ("StartReview", "Start Review"),
        ("Decide", "Decide"),
        ("Escalate", "Escalate"),
        ("AddEvidence", "Add Evidence"),
        ("Withdraw", "Withdraw"),
        ("Reopen", "Reopen"),
        ("PanelVote", "Panel Vote"),
    ]

    id = models.UUIDField(primary_key=True)
    case = models.ForeignKey(
        AdjudicationCase,
        on_delete=models.DO_NOTHING,
        db_column="case_id",
        related_name="actions",
    )
    reviewer_id = models.UUIDField()
    action_type = models.CharField(max_length=20, choices=ACTION_CHOICES)
    comment = models.TextField(null=True, blank=True)
    evidence_added = models.JSONField(default=list)
    taken_at = models.DateTimeField()
    hsm_signature = models.BinaryField()
    public_key_hex = models.TextField()
    prev_action_hash = models.TextField(null=True, blank=True)
    canonical_json = models.TextField()

    class Meta:
        db_table = "eel_reviewer_actions"
        managed = False
        ordering = ["taken_at"]
        verbose_name = "Reviewer Action"
        verbose_name_plural = "Reviewer Actions"

    def __str__(self) -> str:
        return f"{self.action_type} by {self.reviewer_id} on {self.taken_at:%Y-%m-%d %H:%M}"


class PanelVote(models.Model):
    """Mirrors eel_panel_votes."""

    VOTE_CHOICES = [
        ("Approve", "Approve"),
        ("Reject", "Reject"),
        ("Abstain", "Abstain"),
    ]

    id = models.UUIDField(primary_key=True)
    case = models.ForeignKey(
        AdjudicationCase,
        on_delete=models.DO_NOTHING,
        db_column="case_id",
        related_name="votes",
    )
    reviewer_id = models.UUIDField()
    vote = models.CharField(max_length=10, choices=VOTE_CHOICES)
    rationale = models.TextField(null=True, blank=True)
    voted_at = models.DateTimeField()
    hsm_signature = models.BinaryField()
    public_key_hex = models.TextField()

    class Meta:
        db_table = "eel_panel_votes"
        managed = False
        ordering = ["voted_at"]
        verbose_name = "Panel Vote"
        verbose_name_plural = "Panel Votes"

    def __str__(self) -> str:
        return f"{self.vote} by {self.reviewer_id}"


class Reviewer(models.Model):
    """Reviewer profile stored in auth_user extended by this proxy model concept.

    In production this would be backed by Keycloak + a reviewer_profiles table.
    For the console, we use Django's auth.User as the underlying store and
    reference reviewer_id (UUID) from the Rust tables by mapping User.pk -> UUID.
    This model is unmanaged; the auth_user table is Django-managed.
    """

    id = models.UUIDField(primary_key=True)
    tier = models.CharField(max_length=20)
    specialisations = models.JSONField(default=list)
    active_case_count = models.IntegerField(default=0)

    class Meta:
        db_table = "eel_reviewers"
        managed = False
        verbose_name = "Reviewer"
        verbose_name_plural = "Reviewers"

    def __str__(self) -> str:
        return f"Reviewer {self.id} [{self.tier}]"
