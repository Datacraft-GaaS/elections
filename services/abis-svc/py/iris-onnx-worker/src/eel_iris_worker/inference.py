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

import hashlib
import logging
import os
import warnings
from typing import Optional

import numpy as np

logger = logging.getLogger(__name__)

# Environment variables for model paths:
# EEL_IRIS_SEGMENT_ONNX_PATH, EEL_IRIS_ENCODE_ONNX_PATH,
# EEL_IRIS_QUALITY_ONNX_PATH, EEL_IRIS_LENS_ONNX_PATH, EEL_IRIS_EMBED_ONNX_PATH


class OnnxSession:
    """Lazy-loaded ONNX session with hash fallback."""

    def __init__(self, env_var: str, task_name: str) -> None:
        self._env_var = env_var
        self._task_name = task_name
        self._session: Optional[object] = None
        self._loaded = False
        self._load()

    def _load(self) -> None:
        path = os.environ.get(self._env_var)
        if not path:
            warnings.warn(
                f"[eel-iris-worker] {self._env_var} not set; "
                f"{self._task_name} will use hash-based fallback. "
                "Set the env var to point to a real ONNX model.",
                stacklevel=2,
            )
            return
        try:
            import onnxruntime as ort  # type: ignore[import-untyped]

            self._session = ort.InferenceSession(path)
            self._loaded = True
            logger.info("Loaded ONNX model for %s from %s", self._task_name, path)
        except Exception as exc:
            warnings.warn(
                f"[eel-iris-worker] Failed to load ONNX model for {self._task_name}: {exc}. "
                "Using hash-based fallback.",
                stacklevel=2,
            )

    @property
    def is_loaded(self) -> bool:
        return self._loaded

    def run(self, inputs: dict[str, np.ndarray]) -> list[np.ndarray]:
        if not self._loaded or self._session is None:
            raise RuntimeError(
                f"ONNX session for {self._task_name} not loaded; hash fallback should be used instead."
            )

        session = self._session
        assert hasattr(session, "run"), "Expected onnxruntime InferenceSession"
        output_names = [o.name for o in session.get_outputs()]  # type: ignore[union-attr]
        return session.run(output_names, inputs)  # type: ignore[union-attr]


# Module-level sessions (lazy-initialised).
_segment_session: Optional[OnnxSession] = None
_encode_session: Optional[OnnxSession] = None
_quality_session: Optional[OnnxSession] = None
_lens_session: Optional[OnnxSession] = None
_embed_session: Optional[OnnxSession] = None


def get_segment_session() -> OnnxSession:
    global _segment_session
    if _segment_session is None:
        _segment_session = OnnxSession("EEL_IRIS_SEGMENT_ONNX_PATH", "segmentation")
    return _segment_session


def get_encode_session() -> OnnxSession:
    global _encode_session
    if _encode_session is None:
        _encode_session = OnnxSession("EEL_IRIS_ENCODE_ONNX_PATH", "encoding")
    return _encode_session


def get_quality_session() -> OnnxSession:
    global _quality_session
    if _quality_session is None:
        _quality_session = OnnxSession("EEL_IRIS_QUALITY_ONNX_PATH", "quality")
    return _quality_session


def get_lens_session() -> OnnxSession:
    global _lens_session
    if _lens_session is None:
        _lens_session = OnnxSession("EEL_IRIS_LENS_ONNX_PATH", "lens-detect")
    return _lens_session


def get_embed_session() -> OnnxSession:
    global _embed_session
    if _embed_session is None:
        _embed_session = OnnxSession("EEL_IRIS_EMBED_ONNX_PATH", "embedding")
    return _embed_session


def models_loaded() -> dict[str, bool]:
    return {
        "segment": get_segment_session().is_loaded,
        "encode": get_encode_session().is_loaded,
        "quality": get_quality_session().is_loaded,
        "lens": get_lens_session().is_loaded,
        "embed": get_embed_session().is_loaded,
    }


# ---- Hash-based fallback implementations ----


def hash_segment(pixels: np.ndarray) -> dict[str, float]:
    """Deterministic fallback: segment via image statistics."""
    h, w = pixels.shape[:2]
    cx, cy = w / 2.0, h / 2.0
    dark = (pixels < 60).astype(np.uint8)
    pupil_r = float(np.sqrt(dark.sum() / np.pi)) if dark.sum() > 0 else w * 0.12
    limbus_r = pupil_r * 3.0
    return {"cx": cx, "cy": cy, "pupil_r": pupil_r, "limbus_r": limbus_r}


def hash_encode(pixels: np.ndarray) -> tuple[bytes, bytes, float]:
    """Deterministic fallback: SHA-256 based IrisCode."""
    flat = pixels.tobytes()
    code_bytes = bytearray()
    for i in range(8):
        code_bytes.extend(hashlib.sha256(flat + bytes([i])).digest())
    mask_bytes = bytes([0xFF] * 256)
    quality = float(np.std(pixels.astype(np.float32)) / 127.5)
    quality = min(1.0, max(0.0, quality))
    return bytes(code_bytes), mask_bytes, quality


def hash_quality(pixels: np.ndarray) -> dict[str, float]:
    """Deterministic fallback: quality from image statistics."""
    arr = pixels.astype(np.float64)
    lap_y = np.abs(np.diff(np.diff(arr, axis=0), axis=0))
    lap_x = np.abs(np.diff(np.diff(arr, axis=1), axis=1))
    # Crop both to the common interior region before summing.
    r = min(lap_y.shape[0], lap_x.shape[0])
    c = min(lap_y.shape[1], lap_x.shape[1])
    lap = lap_y[:r, :c] + lap_x[:r, :c]
    sharpness = float(min(1.0, lap.mean() / 50.0))
    p10, p90 = float(np.percentile(arr, 10)), float(np.percentile(arr, 90))
    contrast = float((p90 - p10) / 255.0)
    return {
        "sharpness": sharpness,
        "contrast": contrast,
        "dilation": 0.35,
        "gaze_angle_deg": 0.0,
        "occlusion_fraction": 0.1,
        "overall": (sharpness * 0.5 + contrast * 0.5),
    }


def hash_lens(pixels: np.ndarray) -> dict[str, bool | str | float]:
    """Deterministic fallback: no lens detected."""
    return {"has_lens": False, "kind": "none", "confidence": 0.0}


def hash_embed(pixels: np.ndarray) -> list[float]:
    """Deterministic fallback: 256-d L2-normalised hash embedding."""
    flat = pixels.tobytes()
    vec = np.zeros(256, dtype=np.float32)
    for i in range(8):
        chunk = hashlib.sha256(flat + bytes([i])).digest()
        for j, b in enumerate(chunk):
            vec[i * 32 + j] = float(b) - 128.0
    norm = float(np.linalg.norm(vec))
    if norm > 1e-8:
        vec = vec / norm
    return vec.tolist()
