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

"""Tests for HMAC-SHA256 Gen2v2 challenge/response and nonce dedup."""

from __future__ import annotations

import hashlib
import hmac

import pytest

from eel_rfid_reader.hmac_gen2v2 import (
    _nonce_cache,
    build_challenge,
    generate_nonce,
    record_nonce,
    verify_response,
)


# ---------------------------------------------------------------------------
# Known-vector tests
# ---------------------------------------------------------------------------


def test_build_challenge_known_vector() -> None:
    """HMAC-SHA256(key, nonce||epc) must match a hand-computed reference."""
    key = b"test-key-32-bytes-padded-to-fit!!"[:32]
    nonce = bytes.fromhex("00" * 16)
    epc = "deadbeef"
    epc_bytes = bytes.fromhex(epc)
    expected = hmac.new(key, nonce + epc_bytes, hashlib.sha256).digest()
    result = build_challenge(key, nonce, epc)
    assert result == expected


def test_build_challenge_different_keys_differ() -> None:
    key1 = b"key1" + b"\x00" * 28
    key2 = b"key2" + b"\x00" * 28
    nonce = generate_nonce()
    epc = "aabbccdd"
    assert build_challenge(key1, nonce, epc) != build_challenge(key2, nonce, epc)


def test_build_challenge_different_nonces_differ() -> None:
    key = b"\x01" * 32
    epc = "aabbccdd"
    assert build_challenge(key, b"\x00" * 16, epc) != build_challenge(
        key, b"\xff" * 16, epc
    )


def test_verify_response_round_trip() -> None:
    key = b"\x42" * 32
    nonce = generate_nonce()
    epc = "cafebabe"
    challenge = build_challenge(key, nonce, epc)
    assert verify_response(key, nonce.hex(), epc, challenge) is True


def test_verify_response_wrong_bytes() -> None:
    key = b"\x42" * 32
    nonce = generate_nonce()
    epc = "cafebabe"
    assert verify_response(key, nonce.hex(), epc, b"\x00" * 32) is False


def test_verify_response_bad_nonce_hex() -> None:
    key = b"\x42" * 32
    assert verify_response(key, "not-hex", "cafebabe", b"\x00") is False


# ---------------------------------------------------------------------------
# Nonce dedup tests
# ---------------------------------------------------------------------------


def test_nonce_dedup_rejects_replay() -> None:
    _nonce_cache.clear()
    epc = "replay-test-epc"
    nonce_hex = "aa" * 16
    record_nonce(epc, nonce_hex)
    with pytest.raises(ValueError, match="replay"):
        record_nonce(epc, nonce_hex)


def test_nonce_dedup_different_epc_ok() -> None:
    _nonce_cache.clear()
    nonce_hex = "bb" * 16
    record_nonce("epc-1", nonce_hex)
    # Same nonce bytes but different EPC should be accepted.
    record_nonce("epc-2", nonce_hex)


def test_nonce_dedup_different_nonce_ok() -> None:
    _nonce_cache.clear()
    epc = "dedup-epc"
    record_nonce(epc, "cc" * 16)
    record_nonce(epc, "dd" * 16)


def test_generate_nonce_length() -> None:
    n = generate_nonce()
    assert len(n) == 16


def test_generate_nonce_random() -> None:
    """Two calls should produce different nonces with overwhelming probability."""
    assert generate_nonce() != generate_nonce()
