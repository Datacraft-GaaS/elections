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

"""Async upstream client: forwards reader events to eel-inventory-rfid-cover-custody REST API."""

from __future__ import annotations

import logging

import httpx
from uuid6 import uuid7

from .config import cfg
from .models import ReadRequest

_log_ = logging.getLogger(__name__)


def _uuid7str() -> str:
    return str(uuid7())


class UpstreamClient:
    """Forwards RFID read events to the inventory cover-custody service."""

    def __init__(self, base_url: str | None = None, svid: str = "") -> None:
        self._base_url = (base_url or cfg.upstream_url).rstrip("/")
        self._svid = svid
        self._http: httpx.AsyncClient | None = None

    async def start(self) -> None:
        self._http = httpx.AsyncClient(timeout=10.0, base_url=self._base_url)

    async def stop(self) -> None:
        if self._http is not None:
            await self._http.aclose()
            self._http = None

    async def forward_read(
        self, reader_id: str, read: ReadRequest, svid: str = ""
    ) -> dict[str, object]:
        """POST a single read event to /v1/rfid/reads on the upstream service.

        Returns the upstream response JSON or raises httpx.HTTPStatusError.
        """
        assert self._http is not None, "UpstreamClient not started"
        idempotency_key = _uuid7str()
        headers: dict[str, str] = {"Idempotency-Key": idempotency_key}
        active_svid = svid or self._svid
        if active_svid:
            headers[cfg.reader_svid_header] = active_svid

        payload = {
            "reader_id": reader_id,
            "tag_epc": read.tag_epc,
            "tamper_indicator": read.tamper_indicator,
            "rssi_dbm": read.rssi_dbm,
            "read_ts": read.read_ts,
            "idempotency_key": idempotency_key,
        }

        try:
            resp = await self._http.post(
                "/v1/rfid/reads", json=payload, headers=headers
            )
            resp.raise_for_status()
            return dict(resp.json())
        except httpx.HTTPStatusError as exc:
            _log_.error("upstream rfid read forward error: %s", exc)
            raise

    async def forward_batch(
        self, reader_id: str, reads: list[ReadRequest], svid: str = ""
    ) -> list[dict[str, object]]:
        """Forward multiple reads; best-effort, collects errors per-read."""
        results: list[dict[str, object]] = []
        for read in reads:
            try:
                result = await self.forward_read(reader_id, read, svid=svid)
                results.append(result)
            except Exception as exc:  # noqa: BLE001
                _log_.warning("batch forward error for epc=%s: %s", read.tag_epc, exc)
                results.append({"tag_epc": read.tag_epc, "error": str(exc)})
        return results
