import json
from datetime import datetime, timezone

import httpx
import pytest
from pytest_httpserver import HTTPServer

from ifmis.client import IfmisClient
from ifmis.models import GlPostingRequest, GlPostingResponse, IfmisVoucher


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


@pytest.fixture
def ifmis_client(httpserver: HTTPServer) -> IfmisClient:
    transport = httpx.ASGITransport  # not used; httpserver provides real TCP
    client = httpx.AsyncClient(base_url=httpserver.url_for("/"))
    return IfmisClient(base_url=httpserver.url_for("/"), client=client)


async def test_post_gl_success(httpserver: HTTPServer, ifmis_client: IfmisClient):
    now_str = _now()
    response_payload = {
        "ifmis_ref": "IFMIS-2024-0001",
        "voucher_id": "voucher-abc",
        "status": "posted",
        "posted_at": now_str,
    }
    httpserver.expect_request("/api/v1/gl/postings", method="POST").respond_with_json(
        response_payload
    )

    req = GlPostingRequest(
        voucher_id="voucher-abc",
        budget_line_code="PROG-001",
        amount_kes=50_000,
        description="Test posting",
        posting_date=datetime.now(timezone.utc),
        initiated_by="user-1",
        approved_by="user-2",
    )
    resp = await ifmis_client.post_gl(req)
    assert isinstance(resp, GlPostingResponse)
    assert resp.ifmis_ref == "IFMIS-2024-0001"
    assert resp.status == "posted"


async def test_get_voucher_success(httpserver: HTTPServer, ifmis_client: IfmisClient):
    now_str = _now()
    voucher_payload = {
        "ifmis_ref": "IFMIS-2024-0001",
        "voucher_id": "voucher-abc",
        "budget_line_code": "PROG-001",
        "amount_kes": 50_000,
        "description": "Test posting",
        "status": "posted",
        "posting_date": now_str,
        "created_at": now_str,
    }
    httpserver.expect_request(
        "/api/v1/vouchers/IFMIS-2024-0001", method="GET"
    ).respond_with_json(voucher_payload)

    voucher = await ifmis_client.get_voucher("IFMIS-2024-0001")
    assert isinstance(voucher, IfmisVoucher)
    assert voucher.amount_kes == 50_000


async def test_list_vouchers_success(httpserver: HTTPServer, ifmis_client: IfmisClient):
    now_str = _now()
    item = {
        "ifmis_ref": "IFMIS-2024-0002",
        "voucher_id": "voucher-xyz",
        "budget_line_code": "PROG-002",
        "amount_kes": 10_000,
        "description": "Another posting",
        "status": "posted",
        "posting_date": now_str,
        "created_at": now_str,
    }
    httpserver.expect_request("/api/v1/vouchers", method="GET").respond_with_json(
        {"items": [item]}
    )

    vouchers = await ifmis_client.list_vouchers()
    assert len(vouchers) == 1
    assert vouchers[0].ifmis_ref == "IFMIS-2024-0002"


async def test_post_gl_http_error(httpserver: HTTPServer, ifmis_client: IfmisClient):
    httpserver.expect_request("/api/v1/gl/postings", method="POST").respond_with_json(
        {"detail": "unauthorized"}, status=401
    )

    req = GlPostingRequest(
        voucher_id="v1",
        budget_line_code="P1",
        amount_kes=1000,
        description="fail",
        posting_date=datetime.now(timezone.utc),
        initiated_by="u1",
        approved_by="u2",
    )
    with pytest.raises(httpx.HTTPStatusError):
        await ifmis_client.post_gl(req)
