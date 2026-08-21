"""Tests for the FastAPI application."""

from __future__ import annotations

import pytest
from fastapi.testclient import TestClient

from eel_report_deliverers.api import app


@pytest.fixture
def client() -> TestClient:
    return TestClient(app)


def test_health_returns_ok(client: TestClient) -> None:
    resp = client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert data["service"] == "eel-report-deliverers"


def test_deliver_unknown_channel_returns_404(client: TestClient) -> None:
    resp = client.post(
        "/deliver/unknown-channel",
        json={
            "artifact_ref": "http://minio/x",
            "artifact_id": "aid",
            "subscription_id": "sid",
            "format": "pdf",
            "sha256": "a" * 64,
        },
    )
    assert resp.status_code == 404


def test_verify_returns_accepted(client: TestClient) -> None:
    resp = client.get("/verify/some-submission-id")
    assert resp.status_code == 200
    data = resp.json()
    assert data["submission_id"] == "some-submission-id"
    assert data["status"] == "accepted"
    assert data["verified_at"] is not None


def test_deliver_gov_printer_calls_upstream(
    monkeypatch: pytest.MonkeyPatch, client: TestClient
) -> None:
    """Monkeypatch gp.deliver to return a canned receipt."""
    import asyncio
    from datetime import datetime, timezone

    from eel_report_deliverers import api as api_module
    from eel_report_deliverers.models import DeliveryReceipt

    async def fake_deliver(req: object) -> DeliveryReceipt:
        return DeliveryReceipt(
            submission_id="fake-sub-id",
            submitted_at=datetime.now(timezone.utc),
        )

    monkeypatch.setattr(api_module.gp, "deliver", fake_deliver)

    resp = client.post(
        "/deliver/gov-printer",
        json={
            "artifact_ref": "http://minio/x",
            "artifact_id": "aid",
            "subscription_id": "sid",
            "format": "pdf",
            "sha256": "b" * 64,
        },
    )
    assert resp.status_code == 200
    assert resp.json()["submission_id"] == "fake-sub-id"


def test_deliver_kbc_sftp_calls_upstream(
    monkeypatch: pytest.MonkeyPatch, client: TestClient
) -> None:
    from datetime import datetime, timezone

    from eel_report_deliverers import api as api_module
    from eel_report_deliverers.models import DeliveryReceipt

    async def fake_deliver(req: object) -> DeliveryReceipt:
        return DeliveryReceipt(
            remote_path="/kbc/upload/test.tar.gz",
            sha256="c" * 64,
            uploaded_at=datetime.now(timezone.utc),
        )

    monkeypatch.setattr(api_module.kbc, "deliver", fake_deliver)

    resp = client.post(
        "/deliver/kbc-sftp",
        json={
            "artifact_ref": "http://minio/x",
            "artifact_id": "aid",
            "subscription_id": "sid",
            "format": "pdf",
            "sha256": "c" * 64,
        },
    )
    assert resp.status_code == 200
    assert resp.json()["remote_path"] == "/kbc/upload/test.tar.gz"
