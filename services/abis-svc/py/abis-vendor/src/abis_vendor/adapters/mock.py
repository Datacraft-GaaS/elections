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
from hashlib import sha256
from ..models import (
    FaceTemplate,
    FingerprintTemplate,
    HealthStatus,
    IrisTemplate,
    Modality,
    Score,
)
import structlog

_log = structlog.get_logger()


class MockAdapter:
    """Deterministic mock adapter for testing without real biometric hardware."""

    def _log_extract(self, modality: str, image_b64: str) -> None:
        _log.debug("mock_adapter.extract", modality=modality, image_len=len(image_b64))

    def extract_fingerprint(self, image_b64: str) -> FingerprintTemplate:
        self._log_extract("fingerprint", image_b64)
        digest = sha256(image_b64.encode()).hexdigest()
        data = digest[:32]
        return FingerprintTemplate(
            modality=Modality.fingerprint,
            data=data,
            quality=0.85,
            vendor="mock",
        )

    def extract_face(self, image_b64: str) -> FaceTemplate:
        self._log_extract("face", image_b64)
        raw = sha256(image_b64.encode()).digest()
        embedding = [float(b) / 255.0 for b in raw[:16]]
        data = b"mock_face".hex()
        return FaceTemplate(
            modality=Modality.face,
            data=data,
            quality=0.9,
            vendor="mock",
            embedding=embedding,
        )

    def extract_iris(self, image_b64: str) -> IrisTemplate:
        self._log_extract("iris", image_b64)
        data = b"mock_iris".hex()
        return IrisTemplate(
            modality=Modality.iris,
            data=data,
            quality=0.88,
            vendor="mock",
        )

    def match_1x1(
        self,
        probe: FingerprintTemplate | FaceTemplate | IrisTemplate,
        gallery: FingerprintTemplate | FaceTemplate | IrisTemplate,
    ) -> Score:
        score = 1.0 if probe.data == gallery.data else 0.1
        return Score(subject_id=None, score=score)

    def match_1xN(
        self,
        probe: FingerprintTemplate | FaceTemplate | IrisTemplate,
        gallery_batch: list[FingerprintTemplate | FaceTemplate | IrisTemplate],
    ) -> list[Score]:
        return [self.match_1x1(probe, g) for g in gallery_batch]

    def capabilities(self) -> dict[str, object]:
        return {"fingerprint": True, "face": True, "iris": True, "mode": "mock"}

    def health(self) -> HealthStatus:
        return HealthStatus(ok=True, vendor="mock", mode="mock")
