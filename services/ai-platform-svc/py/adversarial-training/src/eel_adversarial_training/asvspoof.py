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

# ASVspoof 2021 dataset loader for voice PAD.
# Citation: Yamagishi et al. "ASVspoof 2021: accelerating progress in spoofed
# and deepfake speech detection" IEEE ASVSPOOF 2021.
#
# Label convention: bonafide=0, spoof=1.

import random
from pathlib import Path

from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7

_MFCC_DIM = 40


def _uuid7str() -> str:
    return str(uuid7())


class AsvSpoofConfig(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    root_dir: str
    n_synthetic: int = 500
    rng_seed: int = 42


class AsvSpoofSample(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    sample_id: str
    label: int
    # 40-dim MFCC-like feature vector.
    audio_features: list[float]


class AsvSpoofLoader:
    def __init__(self, config: AsvSpoofConfig) -> None:
        self._config = config

    def load(self, n: int | None = None) -> list[AsvSpoofSample]:
        root = Path(self._config.root_dir)
        if root.exists() and any(root.iterdir()):
            return self._load_from_disk(n)
        return self._synthetic(n)

    def _load_from_disk(self, n: int | None) -> list[AsvSpoofSample]:
        # Real loader stub: replace with actual ASVspoof file parsing.
        return self._synthetic(n)

    def _synthetic(self, n: int | None) -> list[AsvSpoofSample]:
        count = n if n is not None else self._config.n_synthetic
        rng = random.Random(self._config.rng_seed)
        samples: list[AsvSpoofSample] = []
        for _ in range(count):
            label = rng.randint(0, 1)
            features = [rng.gauss(0.0, 1.0) for _ in range(_MFCC_DIM)]
            samples.append(
                AsvSpoofSample(
                    sample_id=_uuid7str(),
                    label=label,
                    audio_features=features,
                )
            )
        return samples
