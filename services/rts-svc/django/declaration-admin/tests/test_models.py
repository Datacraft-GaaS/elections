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


@pytest.mark.trace("AC-M51-17")
def test_no_delete_on_amendment_admin():
    from declaration_admin.admin import DeclarationAmendmentAdmin
    from declaration_admin.models import DeclarationAmendment
    from django.contrib.admin import site

    # ModelAdmin.__init__ reads `model._meta` to build the permission codenames,
    # so passing None raises before has_delete_permission is ever called -- the
    # assertion would never have run. Pass the model the admin is registered for.
    admin_instance = DeclarationAmendmentAdmin(DeclarationAmendment, site)
    assert not admin_instance.has_delete_permission(None)
