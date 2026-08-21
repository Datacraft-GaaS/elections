# eel-outbox-py

Transactional outbox for eel envelopes. Large payloads are automatically offloaded to MinIO.

## Install

```
uv add eel-outbox-py
```

## Usage

```python
from eel_outbox import emit

async def handle_order(session, envelope):
    await emit(session, envelope)
    await session.commit()
```
