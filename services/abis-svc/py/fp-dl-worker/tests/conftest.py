"""Pytest configuration for fp-dl-worker tests.

Provides a module-level fixture that initialises the FingerprintEmbedder
so tests that call the FastAPI app directly (via ASGITransport without
triggering the lifespan) still see an initialised embedder.
"""

from __future__ import annotations

import pytest

from eel_fp_dl_worker import main as _main_module
from eel_fp_dl_worker.model import FingerprintEmbedder


@pytest.fixture(autouse=True)
def _init_embedder(monkeypatch: pytest.MonkeyPatch) -> None:
    """Patch the module-level embedder so service tests work without lifespan."""
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    monkeypatch.setattr(_main_module, "_embedder", FingerprintEmbedder())
