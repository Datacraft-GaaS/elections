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
    """A reviewer registered in `eel_reviewers` (V023).

    Unmanaged: the Rust migration owns the schema, so the field list is a
    transcription of the table and not a design. Django's `auth_user` is a
    separate store, and `spiffe_id` is the join between them — a reviewer
    authenticates through Django and is identified to the Rust services by
    their SPIFFE id.

    Two columns this model used to carry were not columns at all:

    * `specialisations` is the table `eel_reviewer_specialisations`, one row
      per (reviewer, modality) with a proficiency and its own certification
      date. A JSONField here could not express the modality CHECK, the
      proficiency range, or the per-modality certification.
    * `active_case_count` is derivable from `eel_adjudication_cases_v2`
      (`assigned_to`, open states); `cases_today` is the counter the table
      actually stores, and it is reset by shift rather than by case closure.
    """

    reviewer_id = models.UUIDField(primary_key=True)
    full_name = models.TextField()
    staff_number = models.TextField()
    email_hash = models.BinaryField()
    tier = models.CharField(max_length=20)
    spiffe_id = models.TextField()
    hardware_key_id = models.TextField(null=True, blank=True)
    max_daily_cases = models.IntegerField(default=50)
    cases_today = models.IntegerField(default=0)
    is_on_shift = models.BooleanField(default=False)
    shift_started_at = models.DateTimeField(null=True, blank=True)
    certified_at = models.DateTimeField()
    certification_ref = models.TextField()
    suspended_at = models.DateTimeField(null=True, blank=True)
    suspension_reason = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()

    class Meta:
        db_table = "eel_reviewers"
        managed = False
        verbose_name = "Reviewer"
        verbose_name_plural = "Reviewers"

    def __str__(self) -> str:
        return f"{self.staff_number} [{self.tier}]"


class ReviewerSpecialisation(models.Model):
    """Which modalities a reviewer may adjudicate (V023).

    The table's key is `(reviewer_id, modality)` and it has no `id` column, so
    without `CompositePrimaryKey` Django would synthesise an implicit AutoField
    named `id` and the model would name a column the migration does not create.
    `reviewer_id` is a plain UUID column rather than a `ForeignKey`, which is
    the same choice `ecc-admin`'s telemetry model makes for its composite key:
    the relation is real but the key is the pair, and a FK would add a
    single-column uniqueness the table does not have.
    """

    pk = models.CompositePrimaryKey("reviewer_id", "modality")
    reviewer_id = models.UUIDField()
    modality = models.CharField(max_length=20)
    proficiency = models.SmallIntegerField()
    certified_at = models.DateTimeField()

    class Meta:
        db_table = "eel_reviewer_specialisations"
        managed = False
        verbose_name = "Reviewer Specialisation"
        verbose_name_plural = "Reviewer Specialisations"

    def __str__(self) -> str:
        return f"{self.reviewer_id} / {self.modality} ({self.proficiency})"
