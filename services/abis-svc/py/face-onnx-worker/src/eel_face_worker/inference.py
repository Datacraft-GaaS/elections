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

"""ONNX session pool for face pipeline models.

Model paths are read from environment variables:
  EEL_FACE_DETECT_ONNX_PATH   - RetinaFace / MTCNN detector
  EEL_FACE_LANDMARK_ONNX_PATH - 68-point or 5-point landmark model
  EEL_FACE_EMBED_ONNX_PATH    - ArcFace / AdaFace / CosFace embedder
  EEL_FACE_QUALITY_ONNX_PATH  - ISO 29794-5 quality model

When a path is absent or the file does not exist, the slot falls back to a
hash-based deterministic stub and logs a WARN. This ensures CI passes without
downloading model weights.
"""

from __future__ import annotations

import hashlib
import logging
import os
from typing import Any

import numpy as np

_log = logging.getLogger(__name__)


def _try_load_onnx(env_key: str) -> Any | None:
    """Attempt to load an ONNX session from the env-specified path."""
    path = os.getenv(env_key, "")
    if not path:
        _log.warning("EEL: %s not set; using hash-based stub", env_key)
        return None
    if not os.path.isfile(path):
        _log.warning("EEL: model file not found at %s=%s; using stub", env_key, path)
        return None
    try:
        import onnxruntime as ort  # noqa: PLC0415  # type: ignore[import-untyped]

        sess = ort.InferenceSession(path, providers=["CPUExecutionProvider"])
        _log.info("EEL: loaded ONNX model from %s", path)
        return sess
    except Exception as exc:
        _log.warning("EEL: failed to load %s: %s; using stub", path, exc)
        return None


class _SessionPool:
    def __init__(self) -> None:
        self.detector = _try_load_onnx("EEL_FACE_DETECT_ONNX_PATH")
        self.landmark = _try_load_onnx("EEL_FACE_LANDMARK_ONNX_PATH")
        self.embedder = _try_load_onnx("EEL_FACE_EMBED_ONNX_PATH")
        self.quality = _try_load_onnx("EEL_FACE_QUALITY_ONNX_PATH")

    def models_loaded(self) -> dict[str, bool]:
        return {
            "detector": self.detector is not None,
            "landmark": self.landmark is not None,
            "embedder": self.embedder is not None,
            "quality": self.quality is not None,
        }


_pool: _SessionPool | None = None


def get_pool() -> _SessionPool:
    global _pool
    if _pool is None:
        _pool = _SessionPool()
    return _pool


# ---------------------------------------------------------------------------
# Stub implementations (hash-based, deterministic, no model required)
# ---------------------------------------------------------------------------


def _hash_bytes(data: bytes, n: int) -> bytes:
    """Produce n pseudo-random bytes from data via iterated SHA-256."""
    out = bytearray()
    counter = 0
    while len(out) < n:
        h = hashlib.sha256(data + counter.to_bytes(4, "little")).digest()
        out.extend(h)
        counter += 1
    return bytes(out[:n])


def stub_detect(
    image_bytes: bytes,
    width: int,
    height: int,
    conf_threshold: float,
) -> list[dict[str, Any]]:
    """Hash-based stub detector: returns a single synthetic centred face box."""
    cx, cy = width / 2.0, height / 2.0
    fw, fh = width * 0.4, height * 0.5
    x = cx - fw / 2
    y = cy - fh / 2
    h = hashlib.sha256(image_bytes[:64] if image_bytes else b"empty").digest()
    conf = 0.6 + (h[0] / 255.0) * 0.3
    if conf < conf_threshold:
        return []
    return [
        {
            "bbox": {"x": x, "y": y, "width": fw, "height": fh},
            "confidence": conf,
            "landmarks": {
                "left_eye": (cx - fw * 0.2, cy - fh * 0.1),
                "right_eye": (cx + fw * 0.2, cy - fh * 0.1),
                "nose": (cx, cy + fh * 0.05),
                "left_mouth": (cx - fw * 0.15, cy + fh * 0.25),
                "right_mouth": (cx + fw * 0.15, cy + fh * 0.25),
            },
        }
    ]


def stub_align(image_bytes: bytes, _width: int, _height: int) -> bytes:
    """Stub alignment: returns 112x112x3 uniform grey bytes."""
    return bytes([128] * (112 * 112 * 3))


def stub_embed(aligned_bytes: bytes) -> list[float]:
    """Hash-based stub embedder: deterministic 512-d L2-normalised embedding."""
    raw = _hash_bytes(aligned_bytes or b"empty", 512 * 4)
    floats = np.frombuffer(raw, dtype=np.int32).astype(np.float32) / float(2**31 - 1)
    norm = float(np.linalg.norm(floats))
    if norm < 1e-9:
        floats[0] = 1.0
        norm = 1.0
    floats = floats / norm
    return floats.tolist()


def stub_quality(image_bytes: bytes, width: int, height: int) -> dict[str, float]:
    """Stub quality scorer: returns mid-range scores deterministically."""
    h = hashlib.sha256(image_bytes[:128] if image_bytes else b"empty").digest()
    base = 0.5 + (h[0] / 255.0) * 0.3
    return {
        "sharpness": base,
        "resolution": base,
        "pose_yaw": 0.0,
        "pose_pitch": 0.0,
        "pose_roll": 0.0,
        "illumination": base,
        "uniformity": base,
        "occlusion": 1.0 - base * 0.1,
        "expression": base,
        "gaze": base,
        "overall": base,
    }
