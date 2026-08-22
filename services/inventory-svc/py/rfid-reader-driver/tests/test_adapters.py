"""Tests for adapter implementations: synthetic round-trip, vendor ImportError paths."""

from __future__ import annotations

import pytest

from eel_rfid_reader._internal_key import _derive_key
from eel_rfid_reader.adapters.synthetic import SyntheticAdapter
from eel_rfid_reader.hmac_gen2v2 import _nonce_cache, build_challenge


# ---------------------------------------------------------------------------
# Synthetic adapter: challenge/response round-trip
# ---------------------------------------------------------------------------


async def test_synthetic_round_trip() -> None:
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="test")
    epc = "aabbccddeeff0011"
    nonce_hex, challenge_hex = await adapter.issue_challenge(epc)

    # Compute the expected response using the same key derivation.
    key = _derive_key("test", epc)
    nonce = bytes.fromhex(nonce_hex)
    expected_response = build_challenge(key, nonce, epc)

    result = await adapter.submit_response(
        epc, expected_response, tamper_indicator=False
    )
    assert result.challenge_valid is True
    assert result.tamper_detected is False


async def test_synthetic_wrong_response() -> None:
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="test")
    epc = "1122334455667788"
    await adapter.issue_challenge(epc)
    result = await adapter.submit_response(epc, b"\x00" * 32, tamper_indicator=False)
    assert result.challenge_valid is False


async def test_synthetic_tamper_propagated() -> None:
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="test", tamper_prob=0.0)
    epc = "cafedead00000001"
    nonce_hex, _ = await adapter.issue_challenge(epc)
    key = _derive_key("test", epc)
    nonce = bytes.fromhex(nonce_hex)
    expected = build_challenge(key, nonce, epc)
    result = await adapter.submit_response(epc, expected, tamper_indicator=True)
    assert result.tamper_detected is True


async def test_synthetic_tamper_prob_always() -> None:
    """With tamper_prob=1.0 every response reports tamper."""
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="chaos", tamper_prob=1.0)
    epc = "deadcafebeef0000"
    nonce_hex, _ = await adapter.issue_challenge(epc)
    key = _derive_key("chaos", epc)
    nonce = bytes.fromhex(nonce_hex)
    expected = build_challenge(key, nonce, epc)
    result = await adapter.submit_response(epc, expected, tamper_indicator=False)
    assert result.tamper_detected is True


async def test_synthetic_no_pending_challenge() -> None:
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="test")
    epc = "no-challenge-issued"
    result = await adapter.submit_response(epc, b"\x00" * 32, tamper_indicator=False)
    assert result.challenge_valid is False
    assert "no pending challenge" in result.message


async def test_synthetic_multiple_epcs_isolated() -> None:
    """Challenges for different EPCs should not interfere."""
    _nonce_cache.clear()
    adapter = SyntheticAdapter(seed="multi")
    epc_a = "aaaa000000000001"
    epc_b = "bbbb000000000002"

    nonce_hex_a, _ = await adapter.issue_challenge(epc_a)
    nonce_hex_b, _ = await adapter.issue_challenge(epc_b)

    key_a = _derive_key("multi", epc_a)
    resp_a = build_challenge(key_a, bytes.fromhex(nonce_hex_a), epc_a)
    result_a = await adapter.submit_response(epc_a, resp_a, tamper_indicator=False)
    assert result_a.challenge_valid is True

    key_b = _derive_key("multi", epc_b)
    resp_b = build_challenge(key_b, bytes.fromhex(nonce_hex_b), epc_b)
    result_b = await adapter.submit_response(epc_b, resp_b, tamper_indicator=False)
    assert result_b.challenge_valid is True


# ---------------------------------------------------------------------------
# OSS adapter availability tests (no commercial SDK needed)
# ---------------------------------------------------------------------------


def test_impinj_adapter_uses_llrp_common() -> None:
    """ImpinjLlrpAdapter instantiates without any commercial SDK."""
    from eel_rfid_reader.adapters.impinj_llrp import ImpinjLlrpAdapter
    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient

    adapter = ImpinjLlrpAdapter(host="127.0.0.1", port=5084)
    assert isinstance(adapter._llrp, LlrpCommonClient)
    assert adapter._llrp.host == "127.0.0.1"


def test_zebra_llrp_adapter_instantiates() -> None:
    """ZebraLlrpAdapter instantiates without any commercial SDK."""
    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient
    from eel_rfid_reader.adapters.zebra_llrp import ZebraLlrpAdapter

    adapter = ZebraLlrpAdapter(host="10.0.0.1", port=5084)
    assert isinstance(adapter._llrp, LlrpCommonClient)
    assert adapter._llrp.host == "10.0.0.1"


def test_cipherlab_ble_adapter_instantiates() -> None:
    """CipherLabRS36Adapter instantiates without any commercial SDK."""
    from eel_rfid_reader.adapters.cipherlab_rs36 import CipherLabRS36Adapter

    adapter = CipherLabRS36Adapter(device_address="AA:BB:CC:DD:EE:FF")
    assert adapter._device_address == "AA:BB:CC:DD:EE:FF"
    assert adapter._transport == "ble"
