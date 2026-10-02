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

"""RfidReaderAdapter abstract base: issue_challenge + submit_response."""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import VerifyResult


class RfidReaderAdapter(ABC):
    """Adapter interface that each vendor implementation satisfies."""

    @abstractmethod
    async def issue_challenge(self, tag_epc: str) -> tuple[str, str]:
        """Return (nonce_hex, challenge_hex) for tag_epc.

        The challenge bytes are HMAC-SHA256(reader_key, nonce || epc_bytes).
        """
        ...

    @abstractmethod
    async def submit_response(
        self,
        tag_epc: str,
        response_bytes: bytes,
        tamper_indicator: bool,
    ) -> VerifyResult:
        """Verify a tag's response to a previously issued challenge.

        Args:
                tag_epc: EPC of the responding tag.
                response_bytes: raw bytes the tag returned.
                tamper_indicator: vendor-reported tamper bit (antenna-break / cover-lift).

        Returns:
                VerifyResult with challenge_valid and tamper_detected.
        """
        ...

    async def close(self) -> None:
        """Release vendor SDK resources. Override as needed."""
