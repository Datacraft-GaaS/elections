"""Evidence packaging and transparency-log anchoring (WP-4.1)."""

from __future__ import annotations

import hashlib
import json
import logging
from datetime import datetime, timezone

from .models import EvidencePackage

_log_ = logging.getLogger(__name__)


def uuid7str() -> str:
    from uuid6 import uuid7

    return str(uuid7())


class EvidencePackager:
    """Assemble signed evidence packages and compute tlog hashes."""

    async def assemble(self, case_id: str, content_refs: list[str]) -> EvidencePackage:
        assert case_id, "case_id required"
        assert content_refs, "content_refs must be non-empty"

        pkg_id = uuid7str()
        now = datetime.now(tz=timezone.utc)
        chain_entry = {
            "assembled_at": now.isoformat(),
            "assembled_by": "grievance-channels",
        }
        tlog_hash = await self.hash_to_tlog_stub(pkg_id, case_id, content_refs)

        pkg = EvidencePackage(
            pkg_id=pkg_id,
            case_id=case_id,
            content_refs=content_refs,
            chain_of_custody=chain_entry,
            tlog_hash=tlog_hash,
            assembled_at=now,
        )
        _log_.info(
            "evidence.assemble pkg_id=%s case_id=%s refs=%d tlog_hash=%s",
            pkg_id,
            case_id,
            len(content_refs),
            tlog_hash[:16],
        )
        return pkg

    async def hash_to_tlog_stub(
        self, pkg_id: str, case_id: str, content_refs: list[str]
    ) -> str:
        """Compute sha256 over package metadata. Real impl anchors to tlog P05."""
        payload = json.dumps(
            {
                "pkg_id": pkg_id,
                "case_id": case_id,
                "content_refs": sorted(content_refs),
            },
            sort_keys=True,
        )
        return hashlib.sha256(payload.encode()).hexdigest()
