# eel-pii-py

PII classification wrappers for eel Python services enforcing the ADR-0016 five-tier data model at runtime.

## Purpose in context

Python services handling voter records, biometric data, or personal details wrap those fields in typed PII containers. The containers block accidental serialization of Tier-A/B values and emit an audit record on every reveal. SQLAlchemy type decorators map the containers to Postgres TEXT columns transparently.

## Public API

```python
from eel_pii import (
    Purpose, PIITier, PIIAuditSink,
    PIIRestricted, PIISensitive, PIIOperational, PIIPublic, PIIEphemeral,
    pii_column_type,
    PIIRestrictedType, PIISensitiveType, PIIOperationalType,
    PIIPublicType, PIIEphemeralType,
)
```

| Symbol | Tier | Serde behavior |
|---|---|---|
| `PIIRestricted[T]` | A (national ID, biometric) | raises `PIISerializationError` if serialized |
| `PIISensitive[T]` | B (name, DOB) | raises `PIISerializationError` if serialized |
| `PIIPublic[T]` | C | transparent |
| `PIIOperational[T]` | D | transparent |
| `PIIEphemeral[T]` | E | transparent |

## Layout

```
eel-pii-py/
  src/eel_pii/__init__.py   # all types
  tests/
  pyproject.toml
```

## Usage

```python
from eel_pii import PIIRestricted, PIISensitive, Purpose

class Voter:
    national_id: PIIRestricted[str]
    full_name: PIISensitive[str]

class LogSink:
    async def record(self, subject, purpose, field, tier): ...

voter = Voter()
voter.national_id = PIIRestricted("KE12345678")
voter.full_name = PIISensitive("Jane Mwangi")

sink = LogSink()
nid = await voter.national_id.reveal(Purpose.Identification, sink)
# str(voter.national_id)  =>  "[RESTRICTED]"
```

## SQLAlchemy column mapping

```python
from sqlalchemy import Column
from eel_pii import PIIRestrictedType

class VoterModel(Base):
    __tablename__ = "voters"
    national_id = Column(PIIRestrictedType(), nullable=False)
```

## Dependencies

`pydantic>=2.0`, `sqlalchemy>=2.0`.

## Install

```bash
uv add eel-pii-py
```

## Test

```bash
uv run pytest packages/eel-pii-py/tests -v
uv run pyright packages/eel-pii-py/src
```

## Related docs

- `packages/eel-pii-rs/README.md` - canonical Rust implementation
- `packages/data-dictionary/README.md` - column classification per table
- `docs/adr/0016-data-classification.md` - tier definitions and retention rules

## Limitations

The runtime guard prevents accidental JSON serialization; it does not provide column-level encryption. Tier-A encryption is enforced at the database layer via Postgres column encryption configured by the DBA runbook.
