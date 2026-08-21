"""Batch worker consuming eel.portal.explainer_published.v1 events.

Uses aiokafka if available; falls back to a simple no-op loop with a
clear error log when aiokafka is not installed.
"""

from __future__ import annotations

import asyncio
import logging
import os

log = logging.getLogger(__name__)

TOPIC = "eel.portal.explainer_published.v1"
_BOOTSTRAP = os.environ.get("EEL_KAFKA_BOOTSTRAP", "kafka:9092")


async def run_worker(adapter: object | None = None) -> None:
    """Start the Kafka batch worker.

    Parameters
    ----------
    adapter:
            TTSAdapter instance. If None, resolved from env.
    """
    try:
        from aiokafka import AIOKafkaConsumer  # type: ignore[import]
    except ImportError:
        log.error(
            "aiokafka is not installed. Kafka batch worker disabled. "
            "Install via: pip install aiokafka  (or uv add aiokafka). "
            "Set EEL_KAFKA_BOOTSTRAP to enable."
        )
        return

    from .synth import get_adapter

    if adapter is None:
        adapter = get_adapter()

    consumer = AIOKafkaConsumer(
        TOPIC,
        bootstrap_servers=_BOOTSTRAP,
        group_id="eel-portal-tts",
        auto_offset_reset="earliest",
    )
    log.info("TTS batch worker starting, topic=%s bootstrap=%s", TOPIC, _BOOTSTRAP)
    try:
        await consumer.start()
        async for msg in consumer:
            if msg.value is None:
                continue
            try:
                import json

                event = json.loads(msg.value)
                data = event.get("data", {})
                question_id: str = data.get("question_id", "unknown")
                lang: str = data.get("lang", "en")
                text: str = data.get("text", "")
                if text:
                    mp3 = await adapter.synthesise(text, lang)  # type: ignore[union-attr]
                    log.info(
                        "TTS rendered question_id=%s lang=%s bytes=%d",
                        question_id,
                        lang,
                        len(mp3),
                    )
            except Exception:
                log.exception("Failed to process TTS event offset=%d", msg.offset)
    except asyncio.CancelledError:
        log.info("TTS batch worker cancelled")
    finally:
        await consumer.stop()
        log.info("TTS batch worker stopped")
