"""Env-driven configuration for eel-rfid-reader-driver."""

from __future__ import annotations

import os


class Config:
    """All settings read from environment; defaults are safe for local dev."""

    upstream_url: str = os.environ.get("EEL_RFID_UPSTREAM_URL", "http://localhost:8000")
    adapter: str = os.environ.get("EEL_RFID_ADAPTER", "synthetic")
    hsm_key_id: str = os.environ.get("EEL_RFID_HSM_KEY_ID", "dev-key-1")
    # Synthetic adapter tamper probability [0.0, 1.0]
    synthetic_tamper_prob: float = float(
        os.environ.get("EEL_RFID_SYNTHETIC_TAMPER_PROB", "0.0")
    )
    # Reader SVID for upstream requests
    reader_svid_header: str = os.environ.get("EEL_RFID_SVID_HEADER", "X-Spiffe-Svid")


cfg = Config()
