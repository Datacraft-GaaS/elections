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
from ..models import FaceTemplate, HealthStatus, Modality, Score
import structlog

_log = structlog.get_logger()

try:
    import onnxruntime as _ort  # type: ignore[import-untyped]
    import numpy as _np  # type: ignore[import-untyped]

    ONNX_AVAILABLE = True
except ImportError:
    _ort = None  # type: ignore[assignment]
    _np = None  # type: ignore[assignment]
    ONNX_AVAILABLE = False


class OnnxFaceAdapter:
    """Face adapter using an ONNX inference session for embedding extraction."""

    def __init__(self, model_path: str) -> None:
        if not ONNX_AVAILABLE or _ort is None:
            raise ImportError("onnxruntime not installed; pip install onnxruntime")
        self._session = _ort.InferenceSession(model_path)
        self._model_path = model_path
        _log.info("onnx_face_adapter.loaded", model_path=model_path)

    def _log_extract(self, image_len: int) -> None:
        _log.debug("onnx_face_adapter.extract", image_len=image_len)

    def extract_face(self, image_b64: str) -> FaceTemplate:
        assert _np is not None
        self._log_extract(len(image_b64))
        raw: bytes
        try:
            raw = base64.b64decode(image_b64)
            arr = _np.frombuffer(raw, dtype=_np.uint8).astype(_np.float32)
        except Exception:
            raw = b""
            arr = _np.zeros(512, dtype=_np.float32)

        # Pad or truncate to expected input shape; actual shape depends on model
        input_name = self._session.get_inputs()[0].name
        try:
            embedding_raw = self._session.run(None, {input_name: arr.reshape(1, -1)})[0]
            embedding: list[float] = embedding_raw.flatten().tolist()
        except Exception:
            arr_list = arr.tolist()
            embedding = (
                arr_list[:512]
                if len(arr_list) >= 512
                else (arr_list + [0.0] * (512 - len(arr_list)))
            )

        # L2 normalize
        norm = float(_np.linalg.norm(embedding)) or 1.0
        embedding = [v / norm for v in embedding]

        data_bytes = raw[:32] if len(raw) >= 32 else raw
        data = base64.b64encode(data_bytes).hex()
        return FaceTemplate(
            modality=Modality.face,
            data=data,
            quality=0.9,
            vendor="onnx",
            embedding=embedding,
        )

    def match_1x1(self, probe: FaceTemplate, gallery: FaceTemplate) -> Score:
        assert _np is not None
        if probe.embedding is None or gallery.embedding is None:
            return Score(subject_id=None, score=0.0)
        p = _np.array(probe.embedding, dtype=_np.float32)
        g = _np.array(gallery.embedding, dtype=_np.float32)
        pn = float(_np.linalg.norm(p)) or 1.0
        gn = float(_np.linalg.norm(g)) or 1.0
        score = float(_np.dot(p / pn, g / gn))
        return Score(subject_id=None, score=score)

    def health(self) -> HealthStatus:
        return HealthStatus(ok=True, vendor="onnx", mode="onnx")
