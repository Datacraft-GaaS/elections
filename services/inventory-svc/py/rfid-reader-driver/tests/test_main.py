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

"""Tests for FastAPI endpoints in eel_rfid_reader.main."""

from __future__ import annotations

import os

import pytest
from fastapi.testclient import TestClient


@pytest.fixture(autouse=True)
def _set_synthetic(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("EEL_RFID_ADAPTER", "synthetic")


@pytest.fixture()
def client():  # type: ignore[return]
    import eel_rfid_reader.main as main_mod

    main_mod._readers.clear()
    # Use context manager to trigger lifespan (initialises _adapter).
    with TestClient(main_mod.app) as c:
        yield c


REG_PAYLOAD = {
    "reader_id": "test-reader-001",
    "svid": "spiffe://eel.local/reader/test-reader-001",
    "location": "warehouse-a",
    "reader_type": "synthetic",
    "firmware_version": "1.0.0",
}


def test_health(client: TestClient) -> None:
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"


def test_register_reader(client: TestClient) -> None:
    resp = client.post("/reader/register", json=REG_PAYLOAD)
    assert resp.status_code == 201
    assert resp.json()["reader_id"] == "test-reader-001"


def test_submit_read_requires_registration(client: TestClient) -> None:
    resp = client.post(
        "/reader/unregistered-reader/read",
        json={"tag_epc": "deadbeef", "tamper_indicator": False},
    )
    assert resp.status_code == 404


def test_submit_read_ok(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    resp = client.post(
        "/reader/test-reader-001/read",
        json={"tag_epc": "deadbeef00000001", "tamper_indicator": False},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["tag_epc"] == "deadbeef00000001"
    assert data["accepted"] is True


def test_submit_read_tamper_flag(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    resp = client.post(
        "/reader/test-reader-001/read",
        json={"tag_epc": "tampered0000ffff", "tamper_indicator": True},
    )
    assert resp.status_code == 200
    assert resp.json()["tag_state"] == "tampered"


def test_batch_read_ok(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    reads = [{"tag_epc": f"epc{i:016x}", "tamper_indicator": False} for i in range(5)]
    resp = client.post(
        "/reader/test-reader-001/batch-read",
        json={"reads": reads},
    )
    assert resp.status_code == 200
    results = resp.json()
    assert len(results) == 5
    assert all(r["accepted"] for r in results)


def test_batch_read_limit_1000(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    reads = [{"tag_epc": f"{i:016x}", "tamper_indicator": False} for i in range(1001)]
    resp = client.post(
        "/reader/test-reader-001/batch-read",
        json={"reads": reads},
    )
    assert resp.status_code == 422  # Pydantic max_length validation


def test_issue_challenge(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    resp = client.post(
        "/reader/test-reader-001/challenge",
        json={"tag_epc": "aabbccddeeff0011"},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "nonce_hex" in data
    assert "challenge_hex" in data
    assert len(bytes.fromhex(data["nonce_hex"])) == 16


def test_reader_status(client: TestClient) -> None:
    client.post("/reader/register", json=REG_PAYLOAD)
    resp = client.get("/reader/test-reader-001/status")
    assert resp.status_code == 200
    data = resp.json()
    assert data["healthy"] is True
    assert data["reader_id"] == "test-reader-001"


def test_reader_status_not_found(client: TestClient) -> None:
    resp = client.get("/reader/ghost-reader/status")
    assert resp.status_code == 404


def test_adapter_selection_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """EEL_RFID_ADAPTER=synthetic should produce SyntheticAdapter."""
    monkeypatch.setenv("EEL_RFID_ADAPTER", "synthetic")
    import eel_rfid_reader.main as main_mod
    from eel_rfid_reader.adapters.synthetic import SyntheticAdapter

    adapter = main_mod.build_adapter()
    assert isinstance(adapter, SyntheticAdapter)


def test_adapter_unknown_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    import eel_rfid_reader.config as cfg_mod
    import eel_rfid_reader.main as main_mod

    monkeypatch.setattr(cfg_mod.cfg, "adapter", "nonexistent")
    with pytest.raises(ValueError, match="unknown"):
        main_mod.build_adapter()
