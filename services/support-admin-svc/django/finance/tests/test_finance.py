import pytest
from django.test import TestCase

from finance_app.models import (
    Budget,
    BudgetLine,
    CoBReport,
    PaymentVoucher,
    ProcurementRecord,
    VoucherStatus,
    uuid7str,
)


@pytest.mark.django_db
def test_budget_created_as_draft():
    budget = Budget.objects.create(
        election_id=uuid7str(),
        financial_year="2024/2025",
        total_kes=10_000_000,
    )
    assert budget.status == "draft"
    assert budget.approved_by is None


@pytest.mark.django_db
def test_budget_line_created():
    budget = Budget.objects.create(
        election_id=uuid7str(),
        financial_year="2024/2025",
        total_kes=5_000_000,
    )
    line = BudgetLine.objects.create(
        budget=budget,
        programme_code="PROG-001",
        description="Polling station materials",
        allocated_kes=1_000_000,
    )
    assert line.committed_kes == 0
    assert line.spent_kes == 0


@pytest.mark.django_db
def test_payment_voucher_sod_violation():
    budget = Budget.objects.create(
        election_id=uuid7str(),
        financial_year="2024/2025",
        total_kes=5_000_000,
    )
    line = BudgetLine.objects.create(
        budget=budget,
        programme_code="PROG-002",
        description="Security",
        allocated_kes=500_000,
    )
    proc = ProcurementRecord.objects.create(
        budget_line=line,
        vendor_id=uuid7str(),
        ppra_ref="PPRA-2024-001",
        amount_kes=100_000,
        procurement_method="open_tender",
    )
    initiator = uuid7str()
    voucher = PaymentVoucher.objects.create(
        procurement=proc,
        amount_kes=100_000,
        initiated_by=initiator,
    )
    with pytest.raises(ValueError, match="SoD"):
        voucher.approve(initiator)
    voucher.refresh_from_db()
    assert voucher.status == VoucherStatus.PENDING


@pytest.mark.django_db
def test_payment_voucher_full_workflow():
    budget = Budget.objects.create(
        election_id=uuid7str(),
        financial_year="2024/2025",
        total_kes=5_000_000,
    )
    line = BudgetLine.objects.create(
        budget=budget,
        programme_code="PROG-003",
        description="Printing",
        allocated_kes=200_000,
    )
    proc = ProcurementRecord.objects.create(
        budget_line=line,
        vendor_id=uuid7str(),
        ppra_ref="PPRA-2024-002",
        amount_kes=50_000,
        procurement_method="low_value",
    )
    initiator = uuid7str()
    approver = uuid7str()
    counter = uuid7str()
    voucher = PaymentVoucher.objects.create(
        procurement=proc,
        amount_kes=50_000,
        initiated_by=initiator,
    )
    voucher.approve(approver)
    assert voucher.status == VoucherStatus.APPROVED

    voucher.counter_approve(counter)
    assert voucher.status == VoucherStatus.COUNTER_APPROVED

    voucher.mark_paid("IFMIS-2024-9999")
    assert voucher.status == VoucherStatus.PAID
    assert voucher.ifmis_ref == "IFMIS-2024-9999"


@pytest.mark.django_db
def test_cob_report_created():
    budget = Budget.objects.create(
        election_id=uuid7str(),
        financial_year="2024/2025",
        total_kes=5_000_000,
    )
    report = CoBReport.objects.create(
        budget=budget,
        reporting_period="Q1",
        total_committed=200_000,
        total_spent=150_000,
        variance=50_000,
    )
    assert report.variance == 50_000
