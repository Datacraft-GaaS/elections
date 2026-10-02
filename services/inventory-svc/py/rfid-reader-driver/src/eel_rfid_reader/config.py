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
