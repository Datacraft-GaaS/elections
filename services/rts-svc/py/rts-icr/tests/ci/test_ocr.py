import asyncio

from rts_icr.models import IcrField
from rts_icr.ocr import extract_fields


def test_extract_fields_stub():
    loop = asyncio.get_event_loop()
    fields = loop.run_until_complete(extract_fields("stub://test", "34a"))
    assert len(fields) > 0
    for f in fields:
        assert 0.0 <= f.confidence <= 1.0
