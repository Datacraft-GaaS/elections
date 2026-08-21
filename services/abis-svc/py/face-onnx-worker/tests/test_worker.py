"""Tests for the EEL face ONNX worker.

All tests use the hash-based stubs (no ONNX model files required).
"""

from __future__ import annotations

import base64

import pytest
from fastapi.testclient import TestClient

from eel_face_worker.main import app
from eel_face_worker.inference import stub_detect, stub_embed, stub_quality, stub_align

client = TestClient(app)


def b64_image(width: int = 64, height: int = 64, val: int = 128) -> str:
    pixels = bytes([val] * (width * height * 3))
    return base64.b64encode(pixels).decode()


def b64_aligned() -> str:
    pixels = bytes([100] * (112 * 112 * 3))
    return base64.b64encode(pixels).decode()


# --- /health ---


def test_health_returns_ok() -> None:
    resp = client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert "models_loaded" in data


def test_health_models_loaded_is_dict() -> None:
    resp = client.get("/health")
    loaded = resp.json()["models_loaded"]
    assert isinstance(loaded, dict)
    assert set(loaded.keys()) == {"detector", "landmark", "embedder", "quality"}


# --- /detect ---


def test_detect_returns_200() -> None:
    resp = client.post(
        "/detect",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
            "conf_threshold": 0.3,
            "iou_threshold": 0.4,
        },
    )
    assert resp.status_code == 200


def test_detect_response_has_faces() -> None:
    resp = client.post(
        "/detect",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
            "conf_threshold": 0.3,
        },
    )
    data = resp.json()
    assert "faces" in data
    assert isinstance(data["faces"], list)


def test_detect_high_threshold_may_return_empty() -> None:
    resp = client.post(
        "/detect",
        json={
            "image_b64": b64_image(val=0),
            "width": 64,
            "height": 64,
            "conf_threshold": 1.0,
        },
    )
    assert resp.status_code == 200


def test_detect_invalid_base64_returns_422() -> None:
    resp = client.post(
        "/detect",
        json={
            "image_b64": "not-valid-base64!!!",
            "width": 64,
            "height": 64,
        },
    )
    assert resp.status_code == 422


# --- /align ---


def test_align_returns_200() -> None:
    resp = client.post(
        "/align",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
            "landmarks": {
                "left_eye": [38.0, 51.0],
                "right_eye": [74.0, 51.0],
                "nose": [56.0, 72.0],
                "left_mouth": [42.0, 92.0],
                "right_mouth": [70.0, 92.0],
            },
        },
    )
    assert resp.status_code == 200


def test_align_returns_112x112() -> None:
    resp = client.post(
        "/align",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
            "landmarks": {
                "left_eye": [38.0, 51.0],
                "right_eye": [74.0, 51.0],
                "nose": [56.0, 72.0],
                "left_mouth": [42.0, 92.0],
                "right_mouth": [70.0, 92.0],
            },
        },
    )
    data = resp.json()
    assert data["width"] == 112
    assert data["height"] == 112
    decoded = base64.b64decode(data["aligned_b64"])
    assert len(decoded) == 112 * 112 * 3


# --- /embed ---


def test_embed_returns_512d_embedding() -> None:
    resp = client.post(
        "/embed",
        json={
            "aligned_b64": b64_aligned(),
            "model_family": "arcface",
        },
    )
    assert resp.status_code == 200
    data = resp.json()
    assert len(data["embedding"]) == 512


def test_embed_l2_normalised() -> None:
    resp = client.post(
        "/embed",
        json={
            "aligned_b64": b64_aligned(),
            "model_family": "arcface",
        },
    )
    emb = resp.json()["embedding"]
    norm = sum(x * x for x in emb) ** 0.5
    assert abs(norm - 1.0) < 1e-3, f"norm={norm}"


def test_embed_deterministic() -> None:
    req = {"aligned_b64": b64_aligned(), "model_family": "adaface"}
    a = client.post("/embed", json=req).json()["embedding"]
    b = client.post("/embed", json=req).json()["embedding"]
    assert a == b


def test_embed_different_inputs_different_output() -> None:
    a = client.post(
        "/embed", json={"aligned_b64": b64_aligned(), "model_family": "arcface"}
    ).json()
    b = client.post(
        "/embed",
        json={
            "aligned_b64": base64.b64encode(bytes([200] * (112 * 112 * 3))).decode(),
            "model_family": "arcface",
        },
    ).json()
    assert a["embedding"] != b["embedding"]


def test_embed_invalid_model_family_returns_422() -> None:
    resp = client.post(
        "/embed",
        json={
            "aligned_b64": b64_aligned(),
            "model_family": "unknown",
        },
    )
    assert resp.status_code == 422


# --- /quality ---


def test_quality_returns_200() -> None:
    resp = client.post(
        "/quality",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
        },
    )
    assert resp.status_code == 200


def test_quality_all_fields_present() -> None:
    resp = client.post(
        "/quality",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
        },
    )
    data = resp.json()
    required = {
        "sharpness",
        "resolution",
        "pose_yaw",
        "pose_pitch",
        "pose_roll",
        "illumination",
        "uniformity",
        "occlusion",
        "expression",
        "gaze",
        "overall",
    }
    assert required.issubset(data.keys())


def test_quality_overall_bounded() -> None:
    resp = client.post(
        "/quality",
        json={
            "image_b64": b64_image(),
            "width": 64,
            "height": 64,
        },
    )
    overall = resp.json()["overall"]
    assert 0.0 <= overall <= 1.0, f"overall={overall}"


# --- Stub unit tests ---


def test_stub_embed_deterministic() -> None:
    a = stub_embed(bytes([128] * 100))
    b = stub_embed(bytes([128] * 100))
    assert a == b


def test_stub_embed_length() -> None:
    out = stub_embed(bytes([1] * 50))
    assert len(out) == 512


def test_stub_quality_keys() -> None:
    out = stub_quality(bytes([100] * 200), 64, 64)
    assert "overall" in out
    assert "sharpness" in out


def test_stub_align_length() -> None:
    out = stub_align(bytes([1] * 100), 32, 32)
    assert len(out) == 112 * 112 * 3
