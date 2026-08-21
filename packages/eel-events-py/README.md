# eel-events-py

CloudEvents envelope, Ed25519 signing, schema registry, and consumer middleware for the eel platform.

## Install

```
uv add eel-events-py
```

## Quick start

```python
from eel_events import EventBuilder, DataClass

env = (
    EventBuilder()
    .event_type("eel.order.placed.v1")
    .source("spiffe://example.org/svc/orders")
    .data({"order_id": "abc"})
    .stream_id("...")
    .seq(0)
    .hlc("1700000000000.0.node1")
    .data_class(DataClass.B)
    .build_unsigned()
)
```
