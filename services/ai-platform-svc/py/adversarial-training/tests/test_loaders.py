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

import asyncio

from eel_adversarial_training.asvspoof import AsvSpoofConfig, AsvSpoofLoader
from eel_adversarial_training.dfdc import DfdcConfig, DfdcLoader
from eel_adversarial_training.faceforensics import (
    FaceForensicsConfig,
    FaceForensicsLoader,
)


def test_faceforensics_synthetic_deterministic() -> None:
    cfg = FaceForensicsConfig(root_dir="/nonexistent", rng_seed=7)
    loader = FaceForensicsLoader(cfg)
    a = loader.load(n=20)
    b = loader.load(n=20)
    assert [s.label for s in a] == [s.label for s in b]
    assert [s.frames for s in a] == [s.frames for s in b]


def test_faceforensics_labels() -> None:
    cfg = FaceForensicsConfig(root_dir="/nonexistent", rng_seed=0)
    loader = FaceForensicsLoader(cfg)
    samples = loader.load(n=100)
    for s in samples:
        assert s.label in (0, 1), f"Unexpected label {s.label}"


def test_dfdc_label_distribution() -> None:
    cfg = DfdcConfig(root_dir="/nonexistent", n_synthetic=200, rng_seed=99)
    loader = DfdcLoader(cfg)
    dist = loader.get_label_distribution()
    # Both classes must appear for a 200-sample random draw.
    assert 0 in dist
    assert 1 in dist
    assert dist[0] + dist[1] == 200


def test_asv_spoof_audio_feature_dim() -> None:
    cfg = AsvSpoofConfig(root_dir="/nonexistent", n_synthetic=50, rng_seed=1)
    loader = AsvSpoofLoader(cfg)
    samples = loader.load()
    for s in samples:
        assert len(s.audio_features) == 40, (
            f"Expected 40 features, got {len(s.audio_features)}"
        )


def test_asv_spoof_labels() -> None:
    cfg = AsvSpoofConfig(root_dir="/nonexistent", n_synthetic=100, rng_seed=2)
    loader = AsvSpoofLoader(cfg)
    samples = loader.load()
    for s in samples:
        assert s.label in (0, 1), f"Unexpected label {s.label}"


def test_faceforensics_disk_loading(tmp_path) -> None:
    from PIL import Image

    root = tmp_path / "ffpp"
    manip = root / "manipulated_sequences" / "Deepfakes" / "c23" / "frames"
    orig = root / "original_sequences" / "youtube" / "c23" / "frames"
    manip.mkdir(parents=True)
    orig.mkdir(parents=True)
    for i in range(3):
        Image.new("L", (16, 16), color=200 + i).save(manip / f"fake_{i}.png")
        Image.new("L", (16, 16), color=10 + i).save(orig / f"real_{i}.jpg")

    cfg = FaceForensicsConfig(root_dir=str(root), compression="c23")
    loader = FaceForensicsLoader(cfg)
    samples = loader.load(n=6)
    assert len(samples) == 6
    by_label: dict[int, int] = {0: 0, 1: 0}
    for s in samples:
        frame = s.frames[0]
        assert len(frame) == 8 and all(len(row) == 8 for row in frame)
        vals = [v for row in frame for v in row]
        assert all(0.0 <= v <= 1.0 for v in vals), "frame not normalised to [0, 1]"
        by_label[s.label] += 1
    assert by_label == {0: 3, 1: 3}


def test_faceforensics_disk_compression_fallback(tmp_path) -> None:
    from PIL import Image

    root = tmp_path / "ffpp"
    raw_dir = root / "original_sequences" / "youtube" / "raw" / "frames"
    raw_dir.mkdir(parents=True)
    Image.new("L", (8, 8), color=128).save(raw_dir / "real.png")

    # Configured compression dir (c23) is absent; the raw mirror still loads.
    cfg = FaceForensicsConfig(root_dir=str(root), compression="c23")
    loader = FaceForensicsLoader(cfg)
    samples = loader.load()
    assert len(samples) == 1
    assert samples[0].label == 0
