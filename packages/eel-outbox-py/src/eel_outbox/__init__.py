"""eel-outbox-py: Transactional outbox for eel envelopes."""

from eel_outbox.outbox import MAX_INLINE_BYTES, OutboxSchema, emit

__all__ = ["emit", "MAX_INLINE_BYTES", "OutboxSchema"]
