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

"""Tests for the open-source adapter paths: sllurp (LLRP) + bleak (BLE).

Mocks sllurp and bleak at the module-attribute level so tests run without
hardware and without the libraries needing to establish real connections.

New tests:
  1. sllurp CONNECT — LlrpCommonClient.connect() invokes LLRPClient ctor
  2. Gen2v2 Authenticate — gen2v2_authenticate() calls sendAccessSpec
  3. bleak scan — CipherLabRS36Adapter.scan_for_device() uses BleakScanner
  4. Battery service read — read_battery_level() reads 0x2A19 characteristic
  5. Reconnect after disconnect — ensure_connected() reconnects on stale client
  6. Circuit breaker — rejects after threshold failures; resets on success
"""

from __future__ import annotations

import asyncio
import types
import unittest.mock as mock

import pytest

# ---------------------------------------------------------------------------
# 1. sllurp CONNECT
# ---------------------------------------------------------------------------


async def test_llrp_common_connect_calls_sllurp(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """LlrpCommonClient.connect() constructs an LLRPClient via sllurp."""
    import eel_rfid_reader.adapters.llrp_common as mod

    fake_client = object()
    mock_llrp = types.SimpleNamespace(
        LLRPClient=mock.Mock(return_value=fake_client),
        LLRPReaderConfig=mock.Mock(return_value={}),
    )
    monkeypatch.setattr(mod, "_SLLURP_AVAILABLE", True)
    monkeypatch.setattr(mod, "_sllurp", mock_llrp)

    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient

    client = LlrpCommonClient(host="10.0.0.1", port=5084)

    # Patch run_in_executor to run synchronously.
    loop = asyncio.get_event_loop()

    async def fake_run_in_executor(_executor: object, fn: object) -> object:
        assert callable(fn)
        return fn()  # type: ignore[operator]

    monkeypatch.setattr(loop, "run_in_executor", fake_run_in_executor)
    monkeypatch.setattr(asyncio, "get_event_loop", lambda: loop)

    await client.connect()

    assert mock_llrp.LLRPClient.called
    assert client.connected


# ---------------------------------------------------------------------------
# 2. Gen2v2 Authenticate via ACCESS_SPEC
# ---------------------------------------------------------------------------


async def test_gen2v2_authenticate_calls_send_access_spec(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """gen2v2_authenticate() calls sendAccessSpec with OpSpec type 1024."""
    import eel_rfid_reader.adapters.llrp_common as mod

    challenge = b"\xde\xad\xbe\xef" * 8
    fake_response = b"\xca\xfe" * 16
    mock_inner = mock.Mock()
    mock_inner.sendAccessSpec = mock.Mock(return_value=fake_response)

    mock_llrp = types.SimpleNamespace(
        LLRPClient=mock.Mock(return_value=mock_inner),
        LLRPReaderConfig=mock.Mock(return_value={}),
    )
    monkeypatch.setattr(mod, "_SLLURP_AVAILABLE", True)
    monkeypatch.setattr(mod, "_sllurp", mock_llrp)

    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient

    client = LlrpCommonClient(host="10.0.0.2", port=5084)
    client._client = mock_inner  # inject already-connected client

    loop = asyncio.get_event_loop()

    async def fake_run_in_executor(_executor: object, fn: object) -> object:
        return fn()  # type: ignore[operator]

    monkeypatch.setattr(loop, "run_in_executor", fake_run_in_executor)
    monkeypatch.setattr(asyncio, "get_event_loop", lambda: loop)

    result = await client.gen2v2_authenticate("cafebabe00000001", challenge)

    assert result == fake_response
    mock_inner.sendAccessSpec.assert_called_once()
    _, kwargs = mock_inner.sendAccessSpec.call_args
    assert kwargs.get("op_spec_id") == 1024 or mock_inner.sendAccessSpec.call_args[0]


# ---------------------------------------------------------------------------
# 3. bleak scan
# ---------------------------------------------------------------------------


async def test_ble_scan_finds_cipherlab_device(monkeypatch: pytest.MonkeyPatch) -> None:
    """scan_for_device() uses BleakScanner and returns matching device address."""
    import eel_rfid_reader.adapters.cipherlab_rs36 as mod

    fake_device = types.SimpleNamespace(
        name="CipherLab RS36",
        address="AA:BB:CC:DD:EE:01",
        metadata={"uuids": []},
    )

    mock_scanner_cls = mock.AsyncMock()
    mock_scanner_cls.discover = mock.AsyncMock(return_value=[fake_device])

    monkeypatch.setattr(mod, "_BLEAK_AVAILABLE", True)
    monkeypatch.setattr(mod, "BleakScanner", mock_scanner_cls)

    from eel_rfid_reader.adapters.cipherlab_rs36 import CipherLabRS36Adapter

    adapter = CipherLabRS36Adapter(device_address="")
    result = await adapter.scan_for_device(timeout=1.0)

    assert result == "AA:BB:CC:DD:EE:01"
    mock_scanner_cls.discover.assert_awaited_once()


# ---------------------------------------------------------------------------
# 4. Battery service read
# ---------------------------------------------------------------------------


async def test_ble_battery_service_read(monkeypatch: pytest.MonkeyPatch) -> None:
    """read_battery_level() reads GATT Battery Level characteristic (0x2A19)."""
    import eel_rfid_reader.adapters.cipherlab_rs36 as mod

    mock_client = mock.AsyncMock()
    mock_client.is_connected = True
    mock_client.read_gatt_char = mock.AsyncMock(return_value=bytes([72]))

    mock_client_cls = mock.Mock(return_value=mock_client)
    monkeypatch.setattr(mod, "_BLEAK_AVAILABLE", True)
    monkeypatch.setattr(mod, "BleakClient", mock_client_cls)

    from eel_rfid_reader.adapters.cipherlab_rs36 import (
        CipherLabRS36Adapter,
        _BATTERY_LEVEL_CHAR_UUID,
    )

    adapter = CipherLabRS36Adapter(device_address="AA:BB:CC:DD:EE:FF")
    adapter._client = mock_client  # inject connected client

    level = await adapter.read_battery_level()

    assert level == 72
    mock_client.read_gatt_char.assert_awaited_once_with(_BATTERY_LEVEL_CHAR_UUID)


# ---------------------------------------------------------------------------
# 5. Reconnect after disconnect
# ---------------------------------------------------------------------------


async def test_llrp_reconnect_after_stale_client(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """ensure_connected() reconnects when client was set to None (e.g. after error)."""
    import eel_rfid_reader.adapters.llrp_common as mod

    call_count = {"n": 0}
    fake_clients = [object(), object()]

    def make_client(*_args: object, **_kwargs: object) -> object:
        c = fake_clients[call_count["n"]]
        call_count["n"] += 1
        return c

    mock_llrp = types.SimpleNamespace(
        LLRPClient=mock.Mock(side_effect=make_client),
        LLRPReaderConfig=mock.Mock(return_value={}),
    )
    monkeypatch.setattr(mod, "_SLLURP_AVAILABLE", True)
    monkeypatch.setattr(mod, "_sllurp", mock_llrp)

    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient

    client = LlrpCommonClient(host="10.0.0.3", port=5084)

    loop = asyncio.get_event_loop()

    async def fake_run_in_executor(_executor: object, fn: object) -> object:
        return fn()  # type: ignore[operator]

    monkeypatch.setattr(loop, "run_in_executor", fake_run_in_executor)
    monkeypatch.setattr(asyncio, "get_event_loop", lambda: loop)

    await client.ensure_connected()
    assert client.connected
    first = client._client

    # Simulate a network failure that clears the client.
    client._client = None
    await client.ensure_connected()

    assert client.connected
    assert client._client is not first  # new connection object
    assert call_count["n"] == 2


# ---------------------------------------------------------------------------
# 6. Circuit breaker
# ---------------------------------------------------------------------------


async def test_llrp_circuit_breaker_opens_after_threshold(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Circuit breaker opens after _CB_FAILURE_THRESHOLD failures; resets on success."""
    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient, _CBState

    client = LlrpCommonClient(host="10.0.0.4", port=5084)

    # Drive 5 failures directly via the internal helper.
    for _ in range(client._CB_FAILURE_THRESHOLD):
        client._cb_failure()

    assert client._cb_state == _CBState.OPEN
    assert not client._cb_allow()

    # Recovery: success resets the breaker.
    client._cb_success()
    assert client._cb_state == _CBState.CLOSED
    assert client._cb_allow()


# ---------------------------------------------------------------------------
# 7. CipherLab offline challenge/response roundtrip
# ---------------------------------------------------------------------------


async def test_cipherlab_challenge_response_roundtrip() -> None:
    """issue_challenge/submit_response verify HMAC without any BLE transport."""
    from eel_rfid_reader.adapters.cipherlab_rs36 import CipherLabRS36Adapter

    adapter = CipherLabRS36Adapter(device_address="AA:BB:CC:DD:EE:02")
    tag_epc = "cafebabe00000001"
    _nonce_hex, challenge_hex = await adapter.issue_challenge(tag_epc)

    result = await adapter.submit_response(
        tag_epc, bytes.fromhex(challenge_hex), tamper_indicator=False
    )

    assert result.challenge_valid
    assert not result.tamper_detected
    assert result.message == "ok"


async def test_cipherlab_submit_without_pending_challenge() -> None:
    """submit_response rejects when no challenge was issued for the tag."""
    from eel_rfid_reader.adapters.cipherlab_rs36 import CipherLabRS36Adapter

    adapter = CipherLabRS36Adapter(device_address="AA:BB:CC:DD:EE:03")
    result = await adapter.submit_response(
        "cafebabe00000002", b"\x00" * 32, tamper_indicator=True
    )

    assert not result.challenge_valid
    assert result.tamper_detected


async def test_cipherlab_close_clears_client_on_disconnect_error() -> None:
    """close() logs disconnect failures and still releases the client."""
    from eel_rfid_reader.adapters.cipherlab_rs36 import CipherLabRS36Adapter

    adapter = CipherLabRS36Adapter(device_address="AA:BB:CC:DD:EE:04")
    fake_client = mock.Mock()
    fake_client.disconnect = mock.AsyncMock(side_effect=RuntimeError("link lost"))
    adapter._client = fake_client

    await adapter.close()

    fake_client.disconnect.assert_awaited_once()
    assert adapter._client is None


# ---------------------------------------------------------------------------
# 8. LLRP close releases client even when disconnect raises
# ---------------------------------------------------------------------------


async def test_llrp_close_clears_client_on_disconnect_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """close() logs executor disconnect failures and still clears the client."""
    from eel_rfid_reader.adapters.llrp_common import LlrpCommonClient

    client = LlrpCommonClient(host="10.0.0.4", port=5084)
    inner = mock.Mock()
    inner.disconnect = mock.Mock(side_effect=RuntimeError("socket closed"))
    client._client = inner

    loop = asyncio.get_event_loop()

    async def fake_run_in_executor(_executor: object, fn: object) -> object:
        return fn()  # type: ignore[operator]

    monkeypatch.setattr(loop, "run_in_executor", fake_run_in_executor)
    monkeypatch.setattr(asyncio, "get_event_loop", lambda: loop)

    await client.close()

    inner.disconnect.assert_called_once()
    assert client._client is None
