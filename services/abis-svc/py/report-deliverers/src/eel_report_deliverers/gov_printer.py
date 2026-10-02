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

"""Gov Printer channel: builds Gazette XML envelope and posts to the Gov Printer HTTP endpoint."""

from __future__ import annotations

import hashlib
import os
from datetime import datetime, timezone
from typing import cast

import httpx
from lxml import etree  # type: ignore[import-untyped]
from lxml.builder import E  # type: ignore[import-untyped]

from .models import DeliveryReceipt, DeliveryRequest

# Gazette XML namespace (fictional but stable internal identifier).
GAZETTE_NS = "urn:ke:go:gazette:v1"


def build_gazette_xml(req: DeliveryRequest) -> bytes:
    """Build a Gazette XML submission envelope per the documented template."""
    now = datetime.now(timezone.utc).isoformat()

    root = E.GazetteSubmission(  # type: ignore[operator]
        E.SubmissionMeta(  # type: ignore[operator]
            E.SubmittedAt(now),  # type: ignore[operator]
            E.System("eel-abis"),  # type: ignore[operator]
            E.SubscriptionId(req.subscription_id),  # type: ignore[operator]
        ),
        E.Artefact(  # type: ignore[operator]
            E.ArtifactId(req.artifact_id),  # type: ignore[operator]
            E.Format(req.format),  # type: ignore[operator]
            E.Sha256(req.sha256),  # type: ignore[operator]
            E.StorageRef(req.artifact_ref),  # type: ignore[operator]
        ),
        xmlns=GAZETTE_NS,
    )

    return cast(  # type: ignore[return-value]
        bytes,
        etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True),  # type: ignore[call-overload,unknown-member-type]
    )


def envelope_sha256(xml_bytes: bytes) -> str:
    return hashlib.sha256(xml_bytes).hexdigest()


async def deliver(req: DeliveryRequest) -> DeliveryReceipt:
    """POST the Gazette XML envelope to the Gov Printer endpoint."""
    url = os.environ.get("EEL_GOV_PRINTER_URL", "http://localhost:9080/gazette/submit")
    xml_bytes = build_gazette_xml(req)

    async with httpx.AsyncClient(timeout=60.0) as client:
        resp = await client.post(
            url,
            content=xml_bytes,
            headers={"Content-Type": "application/xml; charset=utf-8"},
        )
        resp.raise_for_status()
        body: dict[str, object] = resp.json()

    return DeliveryReceipt(
        submission_id=cast(str | None, body.get("submission_id")),
        submitted_at=datetime.now(timezone.utc),
    )
