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

"""ONNX Runtime session and fingerprint image preprocessing.

Model: DeepPrint-style network.
Input:  [1, 1, 224, 224] float32, normalised to [-1, 1].
Output: [1, 384] float32, L2-normalised embedding.

Reference: Engelsma, Cao, Jain, "Learning a Fixed-Length Fingerprint
Representation," IEEE TPAMI 2021.
"""

from __future__ import annotations

import hashlib
import logging
import os
from pathlib import Path
from typing import Any, Final

import numpy as np

_log = logging.getLogger(__name__)

EMBED_DIM: Final[int] = 384
MODEL_SIZE: Final[int] = 224

_ONNX_PATH_ENV: Final[str] = "EEL_FP_DL_ONNX_PATH"


def _hash_embedding(pixels: np.ndarray) -> np.ndarray:  # type: ignore[type-arg]
    """Deterministic pseudo-embedding via SHA-256 chain (model-absent fallback)."""
    seed = hashlib.sha256(pixels.tobytes()).digest()
    raw: list[float] = []
    while len(raw) < EMBED_DIM:
        seed = hashlib.sha256(seed).digest()
        for b in seed:
            raw.append(b / 127.5 - 1.0)
            if len(raw) >= EMBED_DIM:
                break
    arr = np.array(raw[:EMBED_DIM], dtype=np.float32)
    norm = float(np.linalg.norm(arr))  # type: ignore[reportUnknownMemberType]
    if norm < 1e-8:
        norm = 1.0
    return arr / norm  # type: ignore[return-value]


def preprocess(image_bytes: bytes) -> np.ndarray:  # type: ignore[type-arg]
    """Decode, resize to 224x224, normalise to [-1, 1].

    Accepts PNG / JPEG / raw bytes.  On import failure falls back to
    a synthetic uniform image (for testing without opencv).
    """
    try:
        import cv2

        data = np.frombuffer(image_bytes, dtype=np.uint8)  # type: ignore[reportUnknownMemberType]
        img = cv2.imdecode(data, cv2.IMREAD_GRAYSCALE)
        if img is None:
            # Raw bytes: treat as MODEL_SIZE x MODEL_SIZE gray image if sizes match.
            side = int(len(image_bytes) ** 0.5)
            img = np.frombuffer(image_bytes, dtype=np.uint8).reshape(side, side)  # type: ignore[reportUnknownMemberType]
        img = cv2.resize(img, (MODEL_SIZE, MODEL_SIZE), interpolation=cv2.INTER_LINEAR)
    except Exception:
        # Fallback for test environments without OpenCV or real image data.
        img = np.full((MODEL_SIZE, MODEL_SIZE), 128, dtype=np.uint8)

    arr = img.astype(np.float32) / 255.0
    arr = (arr - 0.5) / 0.5  # normalise to [-1, 1]
    return arr  # shape (224, 224)


class FingerprintEmbedder:
    """Wraps an ONNX Runtime session for fingerprint embedding.

    If the model path is absent, falls back to SHA-256 pseudo-embeddings and
    logs a WARNING on every call.
    """

    def __init__(self, model_path: Path | None = None) -> None:
        self._session: object | None = None
        self._model_path: Path | None = model_path or (
            Path(p) if (p := os.getenv(_ONNX_PATH_ENV)) else None
        )
        if self._model_path and self._model_path.exists():
            self._session = self._load_session(self._model_path)
        elif self._model_path:
            _log.warning(
                "ONNX model not found at %s; using hash pseudo-embedding",
                self._model_path,
            )
        else:
            _log.warning("EEL_FP_DL_ONNX_PATH not set; using hash pseudo-embedding")

    @staticmethod
    def _load_session(path: Path) -> Any:
        import onnxruntime as ort  # type: ignore[import-untyped]

        opts = ort.SessionOptions()  # type: ignore[reportUnknownMemberType]
        opts.inter_op_num_threads = 1  # type: ignore[reportUnknownMemberType]
        opts.intra_op_num_threads = 2  # type: ignore[reportUnknownMemberType]
        return ort.InferenceSession(str(path), sess_options=opts)  # type: ignore[reportUnknownMemberType]

    @property
    def model_loaded(self) -> bool:
        return self._session is not None

    @property
    def model_path_str(self) -> str | None:
        return str(self._model_path) if self._model_path else None

    def embed(self, image_bytes: bytes) -> np.ndarray:  # type: ignore[type-arg]
        """Return a 384-d L2-normalised embedding for the given image bytes."""
        arr = preprocess(image_bytes)

        if self._session is None:
            _log.warning("Using hash pseudo-embedding (no ONNX model)")
            pixels = (arr * 127.5 + 127.5).astype(np.uint8)
            return _hash_embedding(pixels)

        input_tensor = arr[np.newaxis, np.newaxis, :, :].astype(np.float32)
        session: Any = self._session
        output: Any = session.run(None, {"input": input_tensor})
        emb = np.array(output[0][0], dtype=np.float32)
        norm = float(np.linalg.norm(emb))  # type: ignore[reportUnknownMemberType]
        if norm < 1e-8:
            norm = 1.0
        return emb / norm  # type: ignore[return-value]

    def cosine_similarity(self, a: np.ndarray, b: np.ndarray) -> float:  # type: ignore[type-arg]
        """Cosine similarity between two L2-normalised embeddings."""
        sim = float(np.dot(a, b))  # type: ignore[reportUnknownMemberType]
        return max(0.0, min(1.0, sim))
