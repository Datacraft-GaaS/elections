import asyncio

import pytest

from rts_icr.ocr import OcrUnavailableError, extract_fields


def test_extract_fields_unavailable():
    loop = asyncio.get_event_loop()
    with pytest.raises(OcrUnavailableError):
        loop.run_until_complete(extract_fields("stub://test", "34a"))
