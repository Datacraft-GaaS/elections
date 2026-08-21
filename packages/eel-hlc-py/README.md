# eel-hlc-py

Hybrid Logical Clocks (HLC) for the eel platform.

Wire form: `physical_ms.counter.node` — e.g. `"1723731000123.5.42"`.

## Usage

```python
from eel_hlc import Hlc, HlcClock

clock = HlcClock(node=42)
t1 = clock.now()
t2 = clock.now()
assert t2 > t1

# Parse from wire
h = Hlc.from_str("1723731000123.5.42")
print(str(h))  # "1723731000123.5.42"

# Receive from remote
remote = Hlc(physical_ms=10**13, counter=0, node=7)
t = clock.receive(remote)
```
