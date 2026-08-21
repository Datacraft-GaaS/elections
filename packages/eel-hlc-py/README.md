# eel-hlc-py

Hybrid Logical Clock for eel Python services: wire-compatible with `eel-hlc-rs`.

## Purpose in context

Python workers and Django admin services need causally-consistent timestamps that interoperate with Rust services. This package provides `Hlc` (the parsed timestamp type) and `HlcClock` (stateful issuer). The wire format `physical_ms.counter.node` is identical across all language implementations.

## Public API

```python
from eel_hlc import Hlc, HlcClock, HlcClockProtocol

__all__ = ["Hlc", "HlcClock", "HlcClockProtocol"]
```

| Symbol | Kind | Description |
|---|---|---|
| `Hlc` | dataclass | Parsed HLC: `physical_ms`, `counter`, `node`; `from_str()`, `__str__()`, `__ge__()` |
| `HlcClock` | class | `now() -> Hlc`, `receive(remote: Hlc) -> Hlc` |
| `HlcClockProtocol` | Protocol | Structural typing for clock injection in tests |

## Layout

```
eel-hlc-py/
  src/eel_hlc/__init__.py   # all types
  tests/
  pyproject.toml
```

## Usage

```python
from eel_hlc import HlcClock

clock = HlcClock(node=2)
t1 = clock.now()
t2 = clock.now()
assert t2 >= t1

# Round-trip from wire form received in a CloudEvent envelope
from eel_hlc import Hlc
ts = Hlc.from_str("1723731000123.5.1")
print(str(ts))  # "1723731000123.5.1"
```

## Dependencies

No runtime dependencies. Python >= 3.12.

## Install

```bash
uv add eel-hlc-py
# or from source:
uv pip install -e packages/eel-hlc-py
```

## Test

```bash
uv run pytest packages/eel-hlc-py/tests -v
uv run pyright packages/eel-hlc-py/src
```

## Related docs

- `packages/eel-hlc-rs/README.md` - canonical implementation and wire format spec
- `packages/eel-events-py/README.md` - HLC embedded in every CloudEvent envelope

## Limitations

Thread safety: `HlcClock` uses a `threading.Lock`; share one instance across threads or create one per thread. Monotonicity is guaranteed within a single process; cross-process ordering requires exchanging `Hlc` values via events.
