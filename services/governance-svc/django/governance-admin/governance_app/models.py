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


class AuditRunModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "rla_audit_runs"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"AuditRunModel({self.pk})"


class SampledBallotModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "rla_sampled_ballots"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"SampledBallotModel({self.pk})"


class AuditReportModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "rla_audit_reports"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"AuditReportModel({self.pk})"


class PetitionModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "court_petitions"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"PetitionModel({self.pk})"


class EvidenceBundleModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "court_evidence_bundles"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"EvidenceBundleModel({self.pk})"


class TransmissionRecordModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "legal_transmission_records"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"TransmissionRecordModel({self.pk})"


class ComplianceVerdictModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "legal_compliance_verdicts"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"ComplianceVerdictModel({self.pk})"


class StatutoryReportModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "reporting_statutory_reports"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"StatutoryReportModel({self.pk})"


class OagEngagementModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "oag_engagements"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"OagEngagementModel({self.pk})"


class OagResponseModel(models.Model):
    id = models.UUIDField(primary_key=True)
    status = models.CharField(max_length=64)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "oag_responses"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"OagResponseModel({self.pk})"
