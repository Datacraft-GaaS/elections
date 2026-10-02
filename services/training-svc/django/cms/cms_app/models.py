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

from django.db import models
from uuid6 import uuid7


def uuid7str() -> str:
    return str(uuid7())


class ContentFormat(models.TextChoices):
    ARTICLE = "ARTICLE", "Article"
    VIDEO = "VIDEO", "Video"
    AUDIO = "AUDIO", "Audio"
    PRINT_PDF = "PRINT_PDF", "Print PDF"
    WHATSAPP_CARD = "WHATSAPP_CARD", "WhatsApp Card"


class TopicCategory(models.TextChoices):
    REGISTRATION = "REGISTRATION", "Registration"
    HOW_TO_VOTE = "HOW_TO_VOTE", "How to Vote"
    VOTER_RIGHTS = "VOTER_RIGHTS", "Voter Rights"
    CANDIDATE_CONDUCT = "CANDIDATE_CONDUCT", "Candidate Conduct"
    OBSERVER_RIGHTS = "OBSERVER_RIGHTS", "Observer Rights"
    DISPUTE_RESOLUTION = "DISPUTE_RESOLUTION", "Dispute Resolution"
    POLLING_STATION = "POLLING_STATION", "Polling Station"
    GENERAL = "GENERAL", "General"


class ContentStatus(models.TextChoices):
    DRAFT = "DRAFT", "Draft"
    IN_FACT_CHECK = "IN_FACT_CHECK", "In Fact Check"
    FACT_CHECK_PASSED = "FACT_CHECK_PASSED", "Fact Check Passed"
    IN_APPROVAL = "IN_APPROVAL", "In Approval"
    APPROVED = "APPROVED", "Approved"
    PUBLISHED = "PUBLISHED", "Published"
    ARCHIVED = "ARCHIVED", "Archived"
    EXPIRED = "EXPIRED", "Expired"


class FactCheckDecision(models.TextChoices):
    APPROVED = "APPROVED", "Approved"
    RETURNED = "RETURNED", "Returned"


class DistributionChannel(models.TextChoices):
    M13_APP = "M13_APP", "M13 App"
    M40_MEDIA = "M40_MEDIA", "M40 Media"
    COUNTY_EMAIL = "COUNTY_EMAIL", "County Email"
    WHATSAPP_STORE = "WHATSAPP_STORE", "WhatsApp Store"
    CDN_PUSH = "CDN_PUSH", "CDN Push"


class ContentItem(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    title = models.CharField(max_length=500)
    language = models.CharField(max_length=20)
    format = models.CharField(max_length=30, choices=ContentFormat.choices)
    topic_category = models.CharField(max_length=50, choices=TopicCategory.choices)
    target_audience = models.TextField()
    election_cycle_tag = models.CharField(max_length=50)
    author_id = models.UUIDField()
    current_version_id = models.UUIDField(null=True, blank=True)
    status = models.CharField(
        max_length=30, choices=ContentStatus.choices, default=ContentStatus.DRAFT
    )
    approved_by = models.UUIDField(null=True, blank=True)
    approved_at = models.DateTimeField(null=True, blank=True)
    expiry_date = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        managed = True
        db_table = "training_cms_content_items"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"ContentItem({self.title[:60]}, {self.status})"


class ContentVersion(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    content_item = models.ForeignKey(
        ContentItem, on_delete=models.CASCADE, related_name="versions"
    )
    version_number = models.PositiveIntegerField()
    body_markdown = models.TextField()
    asset_file_id = models.UUIDField(null=True, blank=True)
    parent_version_id = models.UUIDField(null=True, blank=True)
    created_by = models.UUIDField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        managed = True
        db_table = "training_cms_content_versions"
        unique_together = [["content_item", "version_number"]]
        ordering = ["-version_number"]

    def __str__(self) -> str:
        return f"ContentVersion({self.content_item_id}, v{self.version_number})"


class FactCheckReview(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    content_item = models.ForeignKey(
        ContentItem, on_delete=models.CASCADE, related_name="fact_check_reviews"
    )
    version_id = models.UUIDField()
    reviewer_id = models.UUIDField()
    annotations = models.JSONField(default=dict)
    decision = models.CharField(
        max_length=20, choices=FactCheckDecision.choices, null=True, blank=True
    )
    completed_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = True
        db_table = "training_cms_fact_check_reviews"
        ordering = ["-completed_at"]

    def __str__(self) -> str:
        return f"FactCheckReview({self.content_item_id}, {self.decision})"


class DistributionEvent(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid7str, editable=False)
    content_item = models.ForeignKey(
        ContentItem, on_delete=models.CASCADE, related_name="distribution_events"
    )
    channel = models.CharField(max_length=30, choices=DistributionChannel.choices)
    county_code = models.CharField(max_length=10, blank=True)
    dispatched_at = models.DateTimeField(auto_now_add=True)
    quantity = models.PositiveIntegerField(default=0)

    class Meta:
        managed = True
        db_table = "training_cms_distribution_events"
        ordering = ["-dispatched_at"]

    def __str__(self) -> str:
        return f"DistributionEvent({self.content_item_id}, {self.channel})"
