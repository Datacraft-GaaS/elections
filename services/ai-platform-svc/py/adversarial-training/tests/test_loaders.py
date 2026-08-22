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
