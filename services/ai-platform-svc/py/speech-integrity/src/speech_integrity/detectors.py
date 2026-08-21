from __future__ import annotations

from datetime import datetime, timezone

from uuid6 import uuid7

from speech_integrity.views import CibCluster, ContentFlag, ContentItem, DeepfakeResult

_MISINFORMATION_KEYWORDS = {"fake", "hoax", "false", "fabricated", "lie", "lies"}
_HATE_KEYWORDS = {"hate", "slur", "genocide", "exterminate", "vermin"}


def _uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


async def classify_content(item: ContentItem) -> list[ContentFlag]:
    flags: list[ContentFlag] = []
    lower = item.content_text.lower()

    if any(kw in lower for kw in _MISINFORMATION_KEYWORDS):
        flags.append(
            ContentFlag(
                flag_id=_uuid7str(),
                item_id=item.item_id,
                flag_type="misinformation",
                confidence=0.7,
                flagged_at=_now(),
            )
        )

    if any(kw in lower for kw in _HATE_KEYWORDS):
        flags.append(
            ContentFlag(
                flag_id=_uuid7str(),
                item_id=item.item_id,
                flag_type="hate_speech",
                confidence=0.7,
                flagged_at=_now(),
            )
        )

    return flags


async def detect_cib(items: list[ContentItem]) -> list[CibCluster]:
    # Group by platform + rough content-length bucket (stub heuristic)
    buckets: dict[tuple[str, int], list[str]] = {}
    for item in items:
        key = (item.platform, len(item.content_text) // 20)
        buckets.setdefault(key, []).append(item.author_id)

    clusters: list[CibCluster] = []
    for account_ids in buckets.values():
        if len(account_ids) >= 2:
            clusters.append(
                CibCluster(
                    cluster_id=_uuid7str(),
                    account_ids=list(dict.fromkeys(account_ids)),
                    similarity_score=0.8,
                    detected_at=_now(),
                )
            )

    return clusters


async def detect_deepfake(item: ContentItem) -> DeepfakeResult:
    is_deepfake = any("deepfake" in url.lower() for url in item.media_urls)
    return DeepfakeResult(
        result_id=_uuid7str(),
        item_id=item.item_id,
        is_deepfake=is_deepfake,
        confidence=0.9 if is_deepfake else 0.05,
        analyzed_at=_now(),
    )
