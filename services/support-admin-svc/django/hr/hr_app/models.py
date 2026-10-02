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

"""Django models for M55 HR-Payroll (WP-5.6)."""

from django.db import models


class ApplicationStatus(models.TextChoices):
    APPLIED = "applied", "Applied"
    SHORTLISTED = "shortlisted", "Shortlisted"
    SELECTED = "selected", "Selected"
    REJECTED = "rejected", "Rejected"


class PayrollStatus(models.TextChoices):
    DRAFT = "draft", "Draft"
    APPROVED = "approved", "Approved"
    PAID = "paid", "Paid"
    FAILED = "failed", "Failed"


class DeductionType(models.TextChoices):
    PAYE = "paye", "PAYE"
    NSSF = "nssf", "NSSF"
    NHIF = "nhif", "NHIF"
    SHIF = "shif", "SHIF"


class JobApplication(models.Model):
    """A job application from a prospective staff member."""

    id = models.UUIDField(primary_key=True)
    role_id = models.UUIDField()
    applicant_name = models.CharField(max_length=255)
    national_id_hash = models.CharField(max_length=64)
    status = models.CharField(
        max_length=20,
        choices=ApplicationStatus.choices,
        default=ApplicationStatus.APPLIED,
    )
    applied_at = models.DateTimeField()
    training_gate_passed = models.BooleanField(default=False)

    class Meta:
        db_table = "hr_job_application"
        ordering = ["-applied_at"]


class StaffContract(models.Model):
    """Employment contract issued to selected staff."""

    id = models.UUIDField(primary_key=True)
    staff_id = models.UUIDField()
    role = models.CharField(max_length=255)
    station_id = models.UUIDField()
    start_date = models.DateTimeField()
    end_date = models.DateTimeField()
    gross_salary_kes = models.BigIntegerField()
    bank_account_hash = models.CharField(max_length=64)
    deployment_ref = models.UUIDField(null=True, blank=True)

    class Meta:
        db_table = "hr_staff_contract"


class PayrollRun(models.Model):
    """A payroll processing run for an election period.

    Scale note: 250k staff within 4 hours requires parallel batch processing.
    Real implementation fans out to a task queue with bounded concurrency and
    bulk-inserts payslip lines in chunks of ~1000 rows.
    """

    id = models.UUIDField(primary_key=True)
    election_id = models.UUIDField()
    period = models.CharField(max_length=20)
    status = models.CharField(
        max_length=20,
        choices=PayrollStatus.choices,
        default=PayrollStatus.DRAFT,
    )
    total_staff = models.PositiveIntegerField(default=0)
    total_gross_kes = models.BigIntegerField(default=0)
    initiated_by = models.UUIDField()
    approved_by = models.UUIDField(null=True, blank=True)
    started_at = models.DateTimeField()
    completed_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "hr_payroll_run"


class PayslipLine(models.Model):
    """A single staff member's payslip within a PayrollRun."""

    id = models.UUIDField(primary_key=True)
    payroll_run = models.ForeignKey(
        PayrollRun, on_delete=models.CASCADE, related_name="payslip_lines"
    )
    staff_id = models.UUIDField()
    gross_kes = models.BigIntegerField()
    paye_kes = models.BigIntegerField()
    nssf_kes = models.BigIntegerField()
    nhif_kes = models.BigIntegerField()
    net_kes = models.BigIntegerField()

    class Meta:
        db_table = "hr_payslip_line"


class StatutoryDeduction(models.Model):
    """A statutory deduction line for a payslip."""

    id = models.UUIDField(primary_key=True)
    payroll_run = models.ForeignKey(
        PayrollRun, on_delete=models.CASCADE, related_name="statutory_deductions"
    )
    staff_id = models.UUIDField()
    deduction_type = models.CharField(max_length=10, choices=DeductionType.choices)
    amount_kes = models.BigIntegerField()
    reference = models.CharField(max_length=100)

    class Meta:
        db_table = "hr_statutory_deduction"
