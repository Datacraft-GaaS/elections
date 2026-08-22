from __future__ import annotations

# FaceForensics++ dataset loader and adversarial augmentation pipeline.
# Citation: Rossler et al. "FaceForensics++: Learning to Detect Manipulated
# Facial Images" ICCV 2019.

import random
from pathlib import Path
from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7

if TYPE_CHECKING:
    pass


def _uuid7str() -> str:
    return str(uuid7())


class FaceForensicsConfig(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    root_dir: str
    split: str = "train"
    compression: str = "c23"
    n_synthetic: int = 1000
    rng_seed: int = 42


class FaceForensicsSample(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    sample_id: str
    label: int
    # Simplified 8x8 grayscale float frame for test speed.
    frames: list[list[list[float]]]


class FaceForensicsLoader:
    def __init__(self, config: FaceForensicsConfig) -> None:
        self._config = config

    def load(self, n: int | None = None) -> list[FaceForensicsSample]:
        root = Path(self._config.root_dir)
        if root.exists() and any(root.iterdir()):
            return self._load_from_disk(n)
        return self._synthetic(n)

    def _load_from_disk(self, n: int | None) -> list[FaceForensicsSample]:
        # Real loader stub: replace with actual FaceForensics++ file parsing.
        return self._synthetic(n)

    def _synthetic(self, n: int | None) -> list[FaceForensicsSample]:
        count = n if n is not None else self._config.n_synthetic
        rng = random.Random(self._config.rng_seed)
        samples: list[FaceForensicsSample] = []
        for _ in range(count):
            label = rng.randint(0, 1)
            frame: list[list[float]] = [
                [rng.uniform(0.0, 1.0) for _ in range(8)] for _ in range(8)
            ]
            samples.append(
                FaceForensicsSample(
                    sample_id=_uuid7str(),
                    label=label,
                    frames=[frame],
                )
            )
        return samples

    def augment(self, samples: list[FaceForensicsSample]) -> list[FaceForensicsSample]:
        rng = random.Random(self._config.rng_seed)
        augmented: list[FaceForensicsSample] = []
        for s in samples:
            new_frames: list[list[list[float]]] = []
            for frame in s.frames:
                # Random horizontal flip.
                if rng.random() < 0.5:
                    frame = [list(reversed(row)) for row in frame]
                # Brightness perturbation in [0, 1].
                delta = rng.uniform(-0.1, 0.1)
                frame = [[max(0.0, min(1.0, v + delta)) for v in row] for row in frame]
                new_frames.append(frame)
            augmented.append(
                FaceForensicsSample(
                    sample_id=s.sample_id,
                    label=s.label,
                    frames=new_frames,
                )
            )
        return augmented
