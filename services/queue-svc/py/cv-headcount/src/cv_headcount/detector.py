from __future__ import annotations

import hashlib

import numpy as np
from PIL import Image


_FIXTURE_PREFIX = "fixture://"
_IMG_W = 640
_IMG_H = 480


def _make_dummy_image() -> np.ndarray:
    """Generate a deterministic 480x640 RGB image."""
    rng = np.random.default_rng(seed=42)
    return rng.integers(0, 256, (_IMG_H, _IMG_W, 3), dtype=np.uint8)


def _station_seed(station_code: str) -> int:
    digest = hashlib.sha256(station_code.encode()).digest()
    return int.from_bytes(digest[:4], "big")


async def estimate_headcount(station_code: str, image_source: str) -> tuple[int, float]:
    """Return (headcount, confidence).

    For fixture:// sources: loads a synthetic image and returns a
    deterministic count derived from the station_code hash.
    For all other sources: returns (0, 0.0) stub pending real CV integration.
    """
    if not image_source.startswith(_FIXTURE_PREFIX):
        return (0, 0.0)

    img_array = _make_dummy_image()
    _ = Image.fromarray(img_array, mode="RGB")  # validates PIL round-trip

    seed = _station_seed(station_code)
    rng = np.random.default_rng(seed=seed)
    headcount = int(rng.integers(5, 150))
    confidence = float(np.clip(rng.uniform(0.6, 0.95), 0.0, 1.0))
    return (headcount, confidence)
