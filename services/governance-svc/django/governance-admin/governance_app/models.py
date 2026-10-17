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

"""Read-only mirrors of the governance tables.

The canonical data lives in the Rust services; this app renders it. Every model
is `managed = False` so Django never writes DDL.

Two things about these mirrors are worth stating once rather than per model,
because getting either wrong raises on the first query and the error names a
column rather than the mistake:

- The primary keys are `TEXT`, spelled `<entity>_id`, in the migrations. The
  earlier stubs used `id = UUIDField(primary_key=True)` for all ten tables, so
  every one of them queried a column that does not exist and typed a text key
  as a uuid.
- The columns are per table, not a common `status`/`created_at` triple.
  `rla_sampled_ballots` has neither; it has `recorded_at` and no status at all.
"""

from django.db import models


class RealField(models.FloatField):
    """A FloatField that reports itself as `real`.

    `risk_limit` and `risk_achieved` are REAL in the migrations. Django ships no
    single-precision float field, so a FloatField against REAL is permanent
    drift: `makemigrations` would emit an ALTER widening it to double precision,
    on a column holding a risk limit that is compared against a tolerance.
    """

    def db_type(self, connection) -> str:
        return "real"


class AuditRunModel(models.Model):
    run_id = models.TextField(primary_key=True)
    election_id = models.TextField()
    position_code = models.TextField()
    audit_method = models.TextField()
    risk_limit = RealField()
    seed_ref = models.TextField(null=True, blank=True)
    status = models.TextField()
    completed_at = models.DateTimeField(null=True, blank=True)
    sig = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "rla_audit_runs"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"AuditRunModel({self.pk})"


class SampledBallotModel(models.Model):
    ballot_id = models.TextField(primary_key=True)
    run_id = models.TextField()
    station_code = models.TextField()
    batch_id = models.TextField()
    ballot_position = models.IntegerField()
    auditor_interpretation = models.TextField(null=True, blank=True)
    discrepancy = models.BooleanField()
    verdict = models.TextField(null=True, blank=True)
    recorded_at = models.DateTimeField(null=True, blank=True)
    synced = models.BooleanField()

    class Meta:
        managed = False
        db_table = "rla_sampled_ballots"
        ordering = ["-recorded_at"]

    def __str__(self) -> str:
        return f"SampledBallotModel({self.pk})"


class AuditReportModel(models.Model):
    report_id = models.TextField(primary_key=True)
    run_id = models.TextField()
    conclusion = models.TextField()
    risk_achieved = RealField(null=True, blank=True)
    discrepancy_count = models.IntegerField()
    sig = models.TextField(null=True, blank=True)
    published_at = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "rla_audit_reports"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"AuditReportModel({self.pk})"


class PetitionModel(models.Model):
    petition_id = models.TextField(primary_key=True)
    petitioner_hash = models.TextField()
    contest_code = models.TextField()
    grounds = models.TextField()
    status = models.TextField()
    filed_at = models.DateTimeField()
    judgment_at = models.DateTimeField(null=True, blank=True)
    ecourt_ref = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "court_petitions"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"PetitionModel({self.pk})"


class EvidenceBundleModel(models.Model):
    bundle_id = models.TextField(primary_key=True)
    petition_id = models.TextField()
    bundle_version = models.IntegerField()
    item_count = models.IntegerField()
    items = models.JSONField()
    sig = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "court_evidence_bundles"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"EvidenceBundleModel({self.pk})"


class TransmissionRecordModel(models.Model):
    record_id = models.TextField(primary_key=True)
    event_id = models.TextField()
    event_type = models.TextField()
    transmission_step = models.TextField()
    station_code = models.TextField(null=True, blank=True)
    officer_id_hash = models.TextField(null=True, blank=True)
    legal_basis_refs = models.JSONField()
    occurred_at = models.DateTimeField()
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "legal_transmission_records"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"TransmissionRecordModel({self.pk})"


class ComplianceVerdictModel(models.Model):
    verdict_id = models.TextField(primary_key=True)
    record_id = models.TextField()
    rule_id = models.TextField()
    rule_section = models.TextField()
    passed = models.BooleanField()
    failure_reason = models.TextField(null=True, blank=True)
    tlog_hash = models.TextField()
    verified_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "legal_compliance_verdicts"
        ordering = ["-verified_at"]

    def __str__(self) -> str:
        return f"ComplianceVerdictModel({self.pk})"


class StatutoryReportModel(models.Model):
    report_id = models.TextField(primary_key=True)
    report_type = models.TextField()
    status = models.TextField()
    period_from = models.DateField()
    period_to = models.DateField()
    output_url = models.TextField(null=True, blank=True)
    sig = models.TextField(null=True, blank=True)
    generated_at = models.DateTimeField(null=True, blank=True)
    delivered_at = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "reporting_statutory_reports"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"StatutoryReportModel({self.pk})"


class OagEngagementModel(models.Model):
    engagement_id = models.TextField(primary_key=True)
    title = models.TextField()
    scope = models.TextField()
    status = models.TextField()
    oag_ref = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "oag_engagements"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"OagEngagementModel({self.pk})"


class OagResponseModel(models.Model):
    response_id = models.TextField(primary_key=True)
    engagement_id = models.TextField()
    response_text = models.TextField()
    responded_at = models.DateTimeField()
    responder_hash = models.TextField()

    class Meta:
        managed = False
        db_table = "oag_responses"
        ordering = ["-responded_at"]

    def __str__(self) -> str:
        return f"OagResponseModel({self.pk})"
