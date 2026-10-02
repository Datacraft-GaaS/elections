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


class BudgetStatus(models.TextChoices):
    DRAFT = "draft", "Draft"
    APPROVED = "approved", "Approved"
    ACTIVE = "active", "Active"
    CLOSED = "closed", "Closed"


class ProcurementMethod(models.TextChoices):
    OPEN_TENDER = "open_tender", "Open Tender"
    RESTRICTED_TENDER = "restricted_tender", "Restricted Tender"
    DIRECT_PROCUREMENT = "direct_procurement", "Direct Procurement"
    REQUEST_FOR_QUOTATION = "request_for_quotation", "Request for Quotation"
    LOW_VALUE = "low_value", "Low Value Procurement"


class ProcurementStatus(models.TextChoices):
    PENDING = "pending", "Pending"
    AWARDED = "awarded", "Awarded"
    CANCELLED = "cancelled", "Cancelled"


class VoucherStatus(models.TextChoices):
    PENDING = "pending", "Pending"
    APPROVED = "approved", "Approved"
    COUNTER_APPROVED = "counter_approved", "Counter Approved"
    PAID = "paid", "Paid"
    REJECTED = "rejected", "Rejected"


class Budget(models.Model):
    id = models.CharField(max_length=36, primary_key=True, default=uuid7str)
    election_id = models.CharField(max_length=36)
    financial_year = models.CharField(max_length=20)
    total_kes = models.BigIntegerField(default=0)
    approved_by = models.CharField(max_length=36, null=True, blank=True)
    status = models.CharField(
        max_length=20, choices=BudgetStatus.choices, default=BudgetStatus.DRAFT
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"Budget {self.financial_year} ({self.status})"


class BudgetLine(models.Model):
    id = models.CharField(max_length=36, primary_key=True, default=uuid7str)
    budget = models.ForeignKey(Budget, on_delete=models.CASCADE, related_name="lines")
    programme_code = models.CharField(max_length=50)
    description = models.TextField()
    allocated_kes = models.BigIntegerField(default=0)
    committed_kes = models.BigIntegerField(default=0)
    spent_kes = models.BigIntegerField(default=0)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["programme_code"]

    def __str__(self) -> str:
        return f"{self.programme_code}: {self.description[:40]}"


class ProcurementRecord(models.Model):
    id = models.CharField(max_length=36, primary_key=True, default=uuid7str)
    budget_line = models.ForeignKey(
        BudgetLine, on_delete=models.CASCADE, related_name="procurements"
    )
    vendor_id = models.CharField(max_length=36)
    ppra_ref = models.CharField(max_length=100, unique=True)
    amount_kes = models.BigIntegerField(default=0)
    procurement_method = models.CharField(
        max_length=30, choices=ProcurementMethod.choices
    )
    status = models.CharField(
        max_length=20,
        choices=ProcurementStatus.choices,
        default=ProcurementStatus.PENDING,
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"ProcurementRecord {self.ppra_ref}"


class PaymentVoucher(models.Model):
    id = models.CharField(max_length=36, primary_key=True, default=uuid7str)
    procurement = models.ForeignKey(
        ProcurementRecord, on_delete=models.CASCADE, related_name="vouchers"
    )
    amount_kes = models.BigIntegerField(default=0)
    initiated_by = models.CharField(max_length=36)
    approved_by = models.CharField(max_length=36, null=True, blank=True)
    counter_approved_by = models.CharField(max_length=36, null=True, blank=True)
    ifmis_ref = models.CharField(max_length=100, null=True, blank=True)
    status = models.CharField(
        max_length=20, choices=VoucherStatus.choices, default=VoucherStatus.PENDING
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"PaymentVoucher {self.id} ({self.status})"

    def approve(self, approver_id: str) -> None:
        if approver_id == self.initiated_by:
            raise ValueError("approver must differ from initiator (SoD violation)")
        self.approved_by = approver_id
        self.status = VoucherStatus.APPROVED
        self.save(update_fields=["approved_by", "status", "updated_at"])

    def counter_approve(self, counter_approver_id: str) -> None:
        if self.status != VoucherStatus.APPROVED:
            raise ValueError("voucher must be approved before counter-approval")
        self.counter_approved_by = counter_approver_id
        self.status = VoucherStatus.COUNTER_APPROVED
        self.save(update_fields=["counter_approved_by", "status", "updated_at"])

    def mark_paid(self, ifmis_ref: str) -> None:
        if self.status != VoucherStatus.COUNTER_APPROVED:
            raise ValueError("voucher must be counter-approved before payment")
        self.ifmis_ref = ifmis_ref
        self.status = VoucherStatus.PAID
        self.save(update_fields=["ifmis_ref", "status", "updated_at"])


class CoBReport(models.Model):
    id = models.CharField(max_length=36, primary_key=True, default=uuid7str)
    budget = models.ForeignKey(
        Budget, on_delete=models.CASCADE, related_name="cob_reports"
    )
    reporting_period = models.CharField(max_length=20)
    total_committed = models.BigIntegerField(default=0)
    total_spent = models.BigIntegerField(default=0)
    variance = models.BigIntegerField(default=0)
    submitted_at = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ["-created_at"]
        unique_together = [("budget", "reporting_period")]

    def __str__(self) -> str:
        return f"CoB {self.reporting_period} for Budget {self.budget_id}"
