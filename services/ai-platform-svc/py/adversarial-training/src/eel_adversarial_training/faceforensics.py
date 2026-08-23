from __future__ import annotations

# FaceForensics++ dataset loader and adversarial augmentation pipeline.
# Citation: Rossler et al. "FaceForensics++: Learning to Detect Manipulated
# Facial Images" ICCV 2019.

import random
from pathlib import Path

from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7

_FRAME_SIZE = 8
_IMAGE_EXTENSIONS = frozenset({".png", ".jpg", ".jpeg", ".bmp"})
_MANIPULATED_TOP_DIR = "manipulated_sequences"
_ORIGINAL_TOP_DIR = "original_sequences"


def _uuid7str() -> str:
    return str(uuid7())


def _load_frame(path: Path) -> list[list[float]]:
    # Lazy import keeps synthetic-only usage free of the image-decode cost.
    from PIL import Image

    with Image.open(path) as img:
        gray = img.convert("L").resize(
            (_FRAME_SIZE, _FRAME_SIZE), Image.Resampling.BILINEAR
        )
        raw = gray.tobytes()
    return [
        [raw[row * _FRAME_SIZE + col] / 255.0 for col in range(_FRAME_SIZE)]
        for row in range(_FRAME_SIZE)
    ]


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
        """Parse a FaceForensics++ tree on disk.

        Expected layout under root_dir:
          manipulated_sequences/<method>/<compression>/frames/*.png
          original_sequences/<actor>/<compression>/frames/*.jpg

        Images under manipulated_sequences get label 1 (spoof), images under
        original_sequences get label 0 (bonafide). Each image yields one
        sample whose frame is an 8x8 grayscale plane normalised to [0, 1].
        When no image matches the configured compression directory the full
        candidate set is used so partial mirrors still load.
        """
        root = Path(self._config.root_dir)
        candidates: list[tuple[Path, int]] = []
        for top_dir, label in ((_MANIPULATED_TOP_DIR, 1), (_ORIGINAL_TOP_DIR, 0)):
            base = root / top_dir
            if not base.is_dir():
                continue
            for path in sorted(base.rglob("*")):
                if path.is_file() and path.suffix.lower() in _IMAGE_EXTENSIONS:
                    candidates.append((path, label))
        compression = self._config.compression
        preferred = [c for c in candidates if compression in c[0].parts]
        selected = preferred or candidates
        count = n if n is not None else len(selected)
        return [
            FaceForensicsSample(
                sample_id=_uuid7str(),
                label=label,
                frames=[_load_frame(path)],
            )
            for path, label in selected[:count]
        ]

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
