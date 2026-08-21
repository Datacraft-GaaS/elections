"""Unit tests for declaration-admin models (WP-3.3, M51, M63)."""

import pytest

pytestmark = pytest.mark.django_db


def test_declaration_str():
    from declaration_admin.models import Declaration

    d = Declaration(
        contest_id="NAT-PRES-2027", election_id="2027-GENERAL", status="pending"
    )
    assert "NAT-PRES-2027" in str(d)
    assert "pending" in str(d)


def test_reconciliation_hold_str():
    from declaration_admin.models import ReconciliationHold
    import uuid

    h = ReconciliationHold(
        station_code="STN001",
        reconciliation_id=uuid.uuid4(),
        flag_reason="over-assigned",
        raised_at=None,
        result_hold_active=True,
    )
    assert "STN001" in str(h)
    assert "True" in str(h)


def test_no_delete_on_amendment_admin():
    from declaration_admin.admin import DeclarationAmendmentAdmin
    from django.contrib.admin import site

    admin_instance = DeclarationAmendmentAdmin(None, site)
    assert not admin_instance.has_delete_permission(None)
