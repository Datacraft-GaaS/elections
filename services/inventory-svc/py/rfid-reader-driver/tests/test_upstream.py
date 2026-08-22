"""Tests for UpstreamClient using pytest-httpserver."""

from __future__ import annotations

import pytest
from pytest_httpserver import HTTPServer

from eel_rfid_reader.models import ReadRequest
from eel_rfid_reader.upstream import UpstreamClient


@pytest.fixture()
async def client(httpserver: HTTPServer):
    c = UpstreamClient(base_url=httpserver.url_for("/"))
    await c.start()
    yield c
    await c.stop()


async def test_forward_read_success(
    httpserver: HTTPServer, client: UpstreamClient
) -> None:
    httpserver.expect_request("/v1/rfid/reads", method="POST").respond_with_json(
        {"accepted": True, "tag_epc": "deadbeef00000001"}
    )
    req = ReadRequest(tag_epc="deadbeef00000001", tamper_indicator=False)
    result = await client.forward_read("reader-1", req)
    assert result["accepted"] is True


async def test_forward_read_includes_idempotency_key(
    httpserver: HTTPServer, client: UpstreamClient
) -> None:
    """Upstream request must carry Idempotency-Key header."""
    received_headers: dict[str, str] = {}

    def handler(req: object) -> object:  # type: ignore[misc]
        from werkzeug.wrappers import Response

        if hasattr(req, "headers"):
            received_headers.update(
                {k.lower(): v for k, v in req.headers.items()}  # type: ignore[union-attr]
            )
        return Response('{"accepted": true}', content_type="application/json")

    httpserver.expect_request("/v1/rfid/reads", method="POST").respond_with_handler(
        handler  # type: ignore[arg-type]
    )
    req = ReadRequest(tag_epc="aabbccddeeff0011", tamper_indicator=False)
    await client.forward_read("reader-1", req)
    assert "idempotency-key" in received_headers or "Idempotency-Key" in {
        k.lower() for k in received_headers
    }


async def test_forward_read_upstream_error(
    httpserver: HTTPServer, client: UpstreamClient
) -> None:
    httpserver.expect_request("/v1/rfid/reads", method="POST").respond_with_data(
        "internal error", status=500
    )
    req = ReadRequest(tag_epc="badbad0000000001", tamper_indicator=False)
    import httpx

    with pytest.raises(httpx.HTTPStatusError):
        await client.forward_read("reader-err", req)


async def test_forward_batch_partial_error(
    httpserver: HTTPServer, client: UpstreamClient
) -> None:
    """Batch forward collects errors without raising."""
    httpserver.expect_ordered_request(
        "/v1/rfid/reads", method="POST"
    ).respond_with_json({"accepted": True})
    httpserver.expect_ordered_request(
        "/v1/rfid/reads", method="POST"
    ).respond_with_data("error", status=502)

    reads = [
        ReadRequest(tag_epc="ok000000000001", tamper_indicator=False),
        ReadRequest(tag_epc="err00000000002", tamper_indicator=False),
    ]
    results = await client.forward_batch("reader-batch", reads)
    assert len(results) == 2
