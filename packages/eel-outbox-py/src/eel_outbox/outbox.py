"""Transactional outbox for eel envelopes.

Envelopes up to 1 MB are stored inline in the outbox table.
Larger payloads are offloaded to object storage and replaced with a blob reference.
"""

from __future__ import annotations

import logging
from typing import Any

from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

from eel_events import Envelope

logger = logging.getLogger(__name__)

MAX_INLINE_BYTES = 1_000_000  # 1 MB

OutboxSchema = """
CREATE TABLE IF NOT EXISTS outbox (
	id          TEXT        NOT NULL PRIMARY KEY,
	event_type  TEXT        NOT NULL,
	stream_id   TEXT        NOT NULL,
	payload     TEXT,
	blob_ref    TEXT,
	created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
	published_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS outbox_published_at_idx ON outbox (published_at)
	WHERE published_at IS NULL;
"""


async def emit(
    session: AsyncSession,
    envelope: Envelope,
    *,
    minio_client: Any | None = None,
    bucket: str = "eel-events",
) -> None:
    """Write envelope to the outbox table within the current session/transaction.

    Blobs larger than MAX_INLINE_BYTES are offloaded to object storage. The
    caller must commit the session after a successful emit.

    Args:
            session: Active SQLAlchemy async session.
            envelope: Envelope to persist.
            minio_client: aioboto3/minio-compatible async client (required for large payloads).
            bucket: Target object storage bucket name.

    Raises:
            ValueError: Payload exceeds 1 MB and no minio_client was provided.
    """
    payload_bytes = envelope.model_dump_json(by_alias=True).encode()
    blob_ref: str | None = None

    if len(payload_bytes) > MAX_INLINE_BYTES:
        if minio_client is None:
            raise ValueError(
                f"Envelope id={envelope.id} payload is {len(payload_bytes)} bytes "
                f"(>{MAX_INLINE_BYTES}); provide a minio_client to store large payloads."
            )
        blob_ref = await _upload_blob(minio_client, bucket, envelope.id, payload_bytes)
        logger.info("Large envelope id=%s offloaded to %s", envelope.id, blob_ref)
        payload_bytes = b""

    await session.execute(
        text(
            "INSERT INTO outbox (id, event_type, stream_id, payload, blob_ref, created_at)"
            " VALUES (:id, :event_type, :stream_id, :payload, :blob_ref, now())"
        ),
        {
            "id": envelope.id,
            "event_type": envelope.event_type,
            "stream_id": envelope.eel_stream_id,
            "payload": payload_bytes.decode() if payload_bytes else None,
            "blob_ref": blob_ref,
        },
    )


async def _upload_blob(client: Any, bucket: str, event_id: str, data: bytes) -> str:
    """Upload raw bytes to object storage. Returns a minio:// URI."""
    key = f"events/{event_id}.json"
    # Compatible with aioboto3 and minio-py async interfaces.
    await client.put_object(Bucket=bucket, Key=key, Body=data)
    return f"minio://{bucket}/{key}"
