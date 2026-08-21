# eel-pii-py

PII classification wrappers for the eel platform.

Five tiers: A (Restricted), B (Sensitive), C (Operational), D (Public), E (Ephemeral).

## Usage

```python
from eel_pii import PIIRestricted, PIISensitive, Purpose

# Wrap a value — repr/str never leak it
nid = PIIRestricted("123456789")
print(nid)  # <PIIRestricted [REDACTED]>

# Reveal with purpose (and optional async audit sink)
value = nid.reveal(Purpose.LEGAL)

# Pydantic model — serializes as "[REDACTED:A]"
from pydantic import BaseModel

class Person(BaseModel):
    national_id: PIIRestricted[str]
    name: PIISensitive[str]

p = Person(national_id="123", name="Alice")
p.model_dump()  # {"national_id": "[REDACTED:A]", "name": "[REDACTED:B]"}
```
