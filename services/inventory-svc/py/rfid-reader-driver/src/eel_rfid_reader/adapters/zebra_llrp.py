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

"""Zebra FX9600 adapter via LLRP (EPCglobal 1.1, open standard).

Replaces the former ZebraLink commercial SDK dependency with the
open sllurp library (MIT-licensed) via LlrpCommonClient.

The Zebra FX9600 supports LLRP natively; no commercial SDK required.
Frequency: 865-868 MHz (Kenya UHF band, CAK 2022 band plan).
Ingress protection: IP54.
"""

from __future__ import annotations

import hmac as _hmac

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult
from .llrp_common import LlrpCommonClient


class ZebraLlrpAdapter(RfidReaderAdapter):
    """Zebra FX9600 adapter using LLRP 1.1 via sllurp (MIT).

    No commercial SDK required.  Set EEL_ZEBRA_HOST to the reader IP.
    Use the synthetic adapter in CI.
    """

    def __init__(
        self,
        host: str,
        port: int = 5084,
        key_seed: str = "zebra",
    ) -> None:
        assert host, "host must be non-empty"
        self._key_seed = key_seed
        self._pending: dict[str, bytes] = {}
        self._llrp = LlrpCommonClient(host=host, port=port, key_seed=key_seed)

    def _key(self, tag_epc: str) -> bytes:
        return _derive_key(self._key_seed, tag_epc)

    async def issue_challenge(self, tag_epc: str) -> tuple[str, str]:
        nonce = generate_nonce()
        key = self._key(tag_epc)
        challenge = build_challenge(key, nonce, tag_epc)
        nonce_hex = nonce.hex()
        record_nonce(tag_epc, nonce_hex)
        self._pending[tag_epc] = nonce
        return nonce_hex, challenge.hex()

    async def submit_response(
        self,
        tag_epc: str,
        response_bytes: bytes,
        tamper_indicator: bool,
    ) -> VerifyResult:
        nonce = self._pending.pop(tag_epc, None)
        if nonce is None:
            return VerifyResult(
                tag_epc=tag_epc,
                challenge_valid=False,
                tamper_detected=tamper_indicator,
                message="no pending challenge for tag",
            )
        key = self._key(tag_epc)
        expected = build_challenge(key, nonce, tag_epc)
        valid = _hmac.compare_digest(expected, response_bytes)
        return VerifyResult(
            tag_epc=tag_epc,
            challenge_valid=valid,
            tamper_detected=tamper_indicator,
            message="ok" if valid else "hmac mismatch",
        )

    async def close(self) -> None:
        await self._llrp.close()
