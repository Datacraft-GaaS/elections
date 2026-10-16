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

"""Tests for M-Pesa B2C client (WP-5.6)."""


import pytest
from pytest_httpserver import HTTPServer

from mpesa_b2c.client import B2CClient
from mpesa_b2c.models import DisbursementItem


def _make_item(n: int = 1) -> DisbursementItem:
    return DisbursementItem(
        staff_id=f"staff-{n}",
        phone_number=f"2547000000{n:02d}",
        amount_kes=50_000,
    )


def _batch_response(items: list[DisbursementItem]) -> dict:
    return {
        "items": [
            {
                "staff_id": it.staff_id,
                "phone_number": it.phone_number,
                "status": "success",
                "conversation_id": f"conv-{it.staff_id}",
            }
            for it in items
        ]
    }


async def test_disburse_batch_success(httpserver: HTTPServer) -> None:
    items = [_make_item(1), _make_item(2)]
    httpserver.expect_request("/v1/b2c/batch", method="POST").respond_with_json(
        _batch_response(items)
    )

    client = B2CClient("key", "secret", base_url=httpserver.url_for(""))
    result = await client.disburse_batch("batch-001", items)

    assert result.batch_id == "batch-001"
    assert result.total == 2
    assert result.succeeded == 2
    assert result.failed == 0


async def test_disburse_batch_idempotent(httpserver: HTTPServer) -> None:
    items = [_make_item(1)]
    httpserver.expect_request("/v1/b2c/batch", method="POST").respond_with_json(
        _batch_response(items)
    )

    client = B2CClient("key", "secret", base_url=httpserver.url_for(""))
    r1 = await client.disburse_batch("batch-002", items)
    # Second call with same batch_id must not hit the server again.
    r2 = await client.disburse_batch("batch-002", items)

    assert r1.succeeded == 1
    assert r2.items[0].status == "already_submitted"
    # Only one request reached the server.
    assert len(httpserver.log) == 1


async def test_disbursement_item_rejects_zero_amount() -> None:
    with pytest.raises(Exception):
        DisbursementItem(staff_id="s1", phone_number="254700000001", amount_kes=0)


async def test_disbursement_item_rejects_negative_amount() -> None:
    with pytest.raises(Exception):
        DisbursementItem(staff_id="s1", phone_number="254700000001", amount_kes=-100)


async def test_disburse_batch_partial_failure(httpserver: HTTPServer) -> None:
    items = [_make_item(1), _make_item(2)]
    response = {
        "items": [
            {
                "staff_id": "staff-1",
                "phone_number": "254700000001",
                "status": "success",
            },
            {
                "staff_id": "staff-2",
                "phone_number": "254700000002",
                "status": "failed",
                "error": "insufficient funds",
            },
        ]
    }
    httpserver.expect_request("/v1/b2c/batch", method="POST").respond_with_json(
        response
    )

    client = B2CClient("key", "secret", base_url=httpserver.url_for(""))
    result = await client.disburse_batch("batch-003", items)

    assert result.succeeded == 1
    assert result.failed == 1
    failed_item = next(r for r in result.items if r.status == "failed")
    assert failed_item.error == "insufficient funds"
