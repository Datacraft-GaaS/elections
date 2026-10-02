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

from __future__ import annotations
import base64
import os
import sys
import pytest
from abis_vendor.adapters.mock import MockAdapter
from abis_vendor.models import FingerprintTemplate, FaceTemplate, IrisTemplate, Modality

os.environ.setdefault("EEL_VENDOR_MODE", "mock")

_adapter = MockAdapter()
_IMAGE_B64 = base64.b64encode(b"fake_image_data_for_testing").decode()
_IMAGE_B64_ALT = base64.b64encode(b"different_image_data").decode()


def test_mock_extract_fingerprint_deterministic():
    t1 = _adapter.extract_fingerprint(_IMAGE_B64)
    t2 = _adapter.extract_fingerprint(_IMAGE_B64)
    assert t1.data == t2.data
    assert t1.quality == t2.quality
    assert t1.modality == Modality.fingerprint


def test_mock_extract_face_deterministic():
    t1 = _adapter.extract_face(_IMAGE_B64)
    t2 = _adapter.extract_face(_IMAGE_B64)
    assert t1.data == t2.data
    assert t1.embedding == t2.embedding
    assert t1.modality == Modality.face


def test_mock_extract_iris_deterministic():
    t1 = _adapter.extract_iris(_IMAGE_B64)
    t2 = _adapter.extract_iris(_IMAGE_B64)
    assert t1.data == t2.data
    assert t1.modality == Modality.iris


def test_mock_match_1x1_identical_templates():
    t = _adapter.extract_fingerprint(_IMAGE_B64)
    score = _adapter.match_1x1(t, t)
    assert score.score == 1.0


def test_mock_match_1x1_different_templates():
    t1 = _adapter.extract_fingerprint(_IMAGE_B64)
    t2 = _adapter.extract_fingerprint(_IMAGE_B64_ALT)
    score = _adapter.match_1x1(t1, t2)
    assert score.score == pytest.approx(0.1)


def test_mock_match_1xN_empty_gallery():
    t = _adapter.extract_fingerprint(_IMAGE_B64)
    scores = _adapter.match_1xN(t, [])
    assert scores == []


def test_mock_match_1xN_single_match():
    t = _adapter.extract_fingerprint(_IMAGE_B64)
    scores = _adapter.match_1xN(t, [t])
    assert len(scores) == 1
    assert scores[0].score == 1.0


def test_mock_health():
    h = _adapter.health()
    assert h.ok is True
    assert h.vendor == "mock"


def test_onnx_unavailable_raises():
    # Simulate onnxruntime not installed by patching sys.modules
    saved = sys.modules.get("onnxruntime")
    sys.modules["onnxruntime"] = None  # type: ignore[assignment]
    try:
        # Re-import with unavailable flag
        import importlib
        import abis_vendor.adapters.onnx_face as onnx_mod

        # Force ONNX_AVAILABLE to False to test the guard
        original = onnx_mod.ONNX_AVAILABLE
        onnx_mod.ONNX_AVAILABLE = False
        with pytest.raises(ImportError, match="onnxruntime not installed"):
            onnx_mod.OnnxFaceAdapter("nonexistent.onnx")
        onnx_mod.ONNX_AVAILABLE = original
    finally:
        if saved is None:
            sys.modules.pop("onnxruntime", None)
        else:
            sys.modules["onnxruntime"] = saved


def test_py_fingerprint_encode_decode():
    raw = b"some binary image data"
    encoded = base64.b64encode(raw).decode()
    decoded = base64.b64decode(encoded)
    assert decoded == raw


def test_mock_capabilities():
    caps = _adapter.capabilities()
    assert caps["fingerprint"] is True
    assert caps["face"] is True
    assert caps["iris"] is True
    assert "mode" in caps


def test_mock_match_1x1_score_range():
    t1 = _adapter.extract_fingerprint(_IMAGE_B64)
    t2 = _adapter.extract_fingerprint(_IMAGE_B64_ALT)
    score = _adapter.match_1x1(t1, t2)
    assert 0.0 <= score.score <= 1.0
