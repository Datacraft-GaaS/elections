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

"""HMAC-SHA256 challenge/response per EPC Gen2v2 authenticate command.

Nonce management: 5-minute dedup window via in-memory LRU (production plugs to Valkey).
"""

from __future__ import annotations

import hashlib
import hmac
import os
import time
from collections import OrderedDict

# Bytes are the canonical type for nonces and keys internally.
_NONCE_BYTES = 16
_WINDOW_SECONDS = 300  # 5 minutes

# LRU nonce cache: maps (tag_epc, nonce_hex) -> expiry_ts
_nonce_cache: OrderedDict[tuple[str, str], float] = OrderedDict()
_MAX_CACHE = 50_000


def _evict_expired() -> None:
    """Remove entries whose 5-minute window has passed."""
    now = time.monotonic()
    to_delete = [k for k, exp in _nonce_cache.items() if exp <= now]
    for k in to_delete:
        del _nonce_cache[k]


def generate_nonce() -> bytes:
    """Return 16 cryptographically random bytes."""
    return os.urandom(_NONCE_BYTES)


def build_challenge(key: bytes, nonce: bytes, tag_epc: str) -> bytes:
    """Compute HMAC-SHA256(key, nonce || epc_bytes).

    This mirrors the EPC Gen2v2 Authenticate command MAC computation.
    """
    epc_bytes = bytes.fromhex(tag_epc) if _looks_hex(tag_epc) else tag_epc.encode()
    msg = nonce + epc_bytes
    return hmac.new(key, msg, hashlib.sha256).digest()


def verify_response(
    key: bytes,
    nonce_hex: str,
    tag_epc: str,
    response_bytes: bytes,
) -> bool:
    """Return True if response_bytes matches the expected HMAC for this nonce + EPC."""
    try:
        nonce = bytes.fromhex(nonce_hex)
    except ValueError:
        return False
    expected = build_challenge(key, nonce, tag_epc)
    return hmac.compare_digest(expected, response_bytes)


def record_nonce(tag_epc: str, nonce_hex: str) -> None:
    """Record a nonce as used. Raises ValueError if it is a replay within 5 minutes."""
    _evict_expired()
    key = (tag_epc, nonce_hex)
    if key in _nonce_cache:
        raise ValueError(
            f"replay: nonce {nonce_hex!r} for tag {tag_epc!r} already seen"
        )
    if len(_nonce_cache) >= _MAX_CACHE:
        _nonce_cache.popitem(last=False)
    _nonce_cache[key] = time.monotonic() + _WINDOW_SECONDS


def _looks_hex(s: str) -> bool:
    try:
        bytes.fromhex(s)
        return True
    except ValueError:
        return False
