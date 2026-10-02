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

# DeepFake Detection Challenge dataset loader.
# Citation: Dolhansky et al. "The DeepFake Detection Challenge (DFDC) Dataset"
# arXiv 2020.

import random
from pathlib import Path

from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7


def _uuid7str() -> str:
    return str(uuid7())


class DfdcConfig(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    root_dir: str
    n_synthetic: int = 1000
    rng_seed: int = 42


class DfdcSample(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    sample_id: str
    label: int
    metadata: dict[str, str]


class DfdcLoader:
    def __init__(self, config: DfdcConfig) -> None:
        self._config = config

    def load(self, n: int | None = None) -> list[DfdcSample]:
        root = Path(self._config.root_dir)
        if root.exists() and any(root.iterdir()):
            return self._load_from_disk(n)
        return self._synthetic(n)

    def _load_from_disk(self, n: int | None) -> list[DfdcSample]:
        # Real loader stub: replace with actual DFDC file parsing.
        return self._synthetic(n)

    def _synthetic(self, n: int | None) -> list[DfdcSample]:
        count = n if n is not None else self._config.n_synthetic
        rng = random.Random(self._config.rng_seed)
        samples: list[DfdcSample] = []
        for i in range(count):
            label = rng.randint(0, 1)
            samples.append(
                DfdcSample(
                    sample_id=_uuid7str(),
                    label=label,
                    metadata={
                        "split": "train",
                        "index": str(i),
                        "source": "synthetic",
                    },
                )
            )
        return samples

    def get_label_distribution(self) -> dict[int, int]:
        samples = self.load()
        dist: dict[int, int] = {}
        for s in samples:
            dist[s.label] = dist.get(s.label, 0) + 1
        return dist
