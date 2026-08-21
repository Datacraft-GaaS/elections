from __future__ import annotations
import base64
import os
import pytest

os.environ.setdefault("EEL_VENDOR_MODE", "mock")

from fastapi.testclient import TestClient
from abis_vendor.service import app

_IMAGE_B64 = base64.b64encode(b"test_image_bytes_for_service_tests").decode()


@pytest.fixture(scope="module")
def client():
    with TestClient(app) as c:
        yield c


def test_health_returns_ok(client: TestClient):
    resp = client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["ok"] is True


def test_extract_fingerprint_mock(client: TestClient):
    resp = client.post(
        "/extract/fingerprint",
        json={"modality": "fingerprint", "image_b64": _IMAGE_B64},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "template" in data
    assert data["template"]["modality"] == "fingerprint"


def test_extract_face_mock(client: TestClient):
    resp = client.post(
        "/extract/face",
        json={"modality": "face", "image_b64": _IMAGE_B64},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["template"]["modality"] == "face"


def test_extract_iris_mock(client: TestClient):
    resp = client.post(
        "/extract/iris",
        json={"modality": "iris", "image_b64": _IMAGE_B64},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["template"]["modality"] == "iris"


def test_match_1x1_mock(client: TestClient):
    # First extract a template
    resp = client.post(
        "/extract/fingerprint",
        json={"modality": "fingerprint", "image_b64": _IMAGE_B64},
    )
    template = resp.json()["template"]
    body = {"probe": template, "gallery": template}
    resp2 = client.post("/match/1x1", json=body)
    assert resp2.status_code == 200
    data = resp2.json()
    assert len(data["scores"]) == 1
    assert data["scores"][0]["score"] == pytest.approx(1.0)


def test_match_1xN_mock(client: TestClient):
    resp = client.post(
        "/extract/fingerprint",
        json={"modality": "fingerprint", "image_b64": _IMAGE_B64},
    )
    template = resp.json()["template"]
    body = {"probe": template, "gallery_batch": [template, template]}
    resp2 = client.post("/match/1xN", json=body)
    assert resp2.status_code == 200
    data = resp2.json()
    assert len(data["scores"]) == 2


def test_modality_mismatch(client: TestClient):
    resp = client.post(
        "/extract/fingerprint",
        json={"modality": "face", "image_b64": _IMAGE_B64},
    )
    assert resp.status_code == 422


def test_invalid_json(client: TestClient):
    resp = client.post(
        "/extract/fingerprint",
        content=b"not json at all",
        headers={"content-type": "application/json"},
    )
    assert resp.status_code == 422
