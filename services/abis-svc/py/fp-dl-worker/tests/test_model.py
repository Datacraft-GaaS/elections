"""Tests for FingerprintEmbedder and preprocessing.

All tests use the deterministic hash-based pseudo-embedding path
(no ONNX model file required).
"""

from __future__ import annotations

import base64
import hashlib

import numpy as np
import pytest

from eel_fp_dl_worker.model import (
    EMBED_DIM,
    MODEL_SIZE,
    FingerprintEmbedder,
    _hash_embedding,
    preprocess,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _solid_image(v: int = 128) -> bytes:
    """Synthetic 64x64 grayscale image as raw bytes."""
    return bytes([v] * (64 * 64))


def _stripe_image() -> bytes:
    """Synthetic 64x64 stripe pattern."""
    pix = []
    for y in range(64):
        row_val = 30 if y % 10 < 5 else 200
        pix.extend([row_val] * 64)
    return bytes(pix)


# ---------------------------------------------------------------------------
# preprocess
# ---------------------------------------------------------------------------


def test_preprocess_output_shape() -> None:
    img = _solid_image()
    arr = preprocess(img)
    assert arr.shape == (MODEL_SIZE, MODEL_SIZE), (
        f"expected ({MODEL_SIZE}, {MODEL_SIZE}), got {arr.shape}"
    )


def test_preprocess_values_in_minus1_to_1() -> None:
    img = _solid_image(128)
    arr = preprocess(img)
    assert arr.min() >= -1.0 - 1e-5
    assert arr.max() <= 1.0 + 1e-5


def test_preprocess_all_white_near_plus1() -> None:
    arr = preprocess(_solid_image(255))
    assert arr.mean() > 0.9, (
        f"all-white image should normalise near +1, got mean={arr.mean()}"
    )


def test_preprocess_all_black_near_minus1() -> None:
    arr = preprocess(_solid_image(0))
    assert arr.mean() < -0.9, (
        f"all-black image should normalise near -1, got mean={arr.mean()}"
    )


# ---------------------------------------------------------------------------
# _hash_embedding
# ---------------------------------------------------------------------------


def test_hash_embedding_correct_dim() -> None:
    pixels = np.zeros((MODEL_SIZE, MODEL_SIZE), dtype=np.uint8)
    emb = _hash_embedding(pixels)
    assert emb.shape == (EMBED_DIM,)


def test_hash_embedding_unit_norm() -> None:
    pixels = np.full((MODEL_SIZE, MODEL_SIZE), 128, dtype=np.uint8)
    emb = _hash_embedding(pixels)
    norm = float(np.linalg.norm(emb))
    assert abs(norm - 1.0) < 1e-4, f"embedding norm={norm} not unit"


def test_hash_embedding_deterministic() -> None:
    pixels = np.full((MODEL_SIZE, MODEL_SIZE), 100, dtype=np.uint8)
    e1 = _hash_embedding(pixels)
    e2 = _hash_embedding(pixels)
    assert np.allclose(e1, e2)


def test_hash_embedding_different_inputs() -> None:
    p1 = np.full((MODEL_SIZE, MODEL_SIZE), 50, dtype=np.uint8)
    p2 = np.full((MODEL_SIZE, MODEL_SIZE), 200, dtype=np.uint8)
    e1 = _hash_embedding(p1)
    e2 = _hash_embedding(p2)
    assert not np.allclose(e1, e2)


# ---------------------------------------------------------------------------
# FingerprintEmbedder (no model path)
# ---------------------------------------------------------------------------


def test_embedder_model_not_loaded_without_path(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    assert not emb.model_loaded


def test_embedder_returns_correct_dim(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    result = emb.embed(_solid_image())
    assert result.shape == (EMBED_DIM,)


def test_embedder_result_unit_norm(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    result = emb.embed(_solid_image())
    norm = float(np.linalg.norm(result))
    assert abs(norm - 1.0) < 1e-4


def test_embedder_deterministic(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    img = _stripe_image()
    e1 = emb.embed(img)
    e2 = emb.embed(img)
    assert np.allclose(e1, e2)


# ---------------------------------------------------------------------------
# cosine_similarity
# ---------------------------------------------------------------------------


def test_cosine_identical(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    img = _stripe_image()
    e = emb.embed(img)
    sim = emb.cosine_similarity(e, e)
    assert abs(sim - 1.0) < 1e-4


def test_cosine_bounded(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    e1 = emb.embed(_solid_image(10))
    e2 = emb.embed(_solid_image(240))
    sim = emb.cosine_similarity(e1, e2)
    assert 0.0 <= sim <= 1.0


def test_cosine_different_images(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    emb = FingerprintEmbedder()
    e1 = emb.embed(_solid_image(0))
    e2 = emb.embed(_solid_image(255))
    sim = emb.cosine_similarity(e1, e2)
    assert sim < 0.5, f"very different images cosine={sim} unexpectedly high"
