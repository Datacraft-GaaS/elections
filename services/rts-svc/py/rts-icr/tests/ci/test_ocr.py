# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import asyncio

import pytest

from rts_icr.ocr import OcrUnavailableError, extract_fields


def test_extract_fields_unavailable():
    loop = asyncio.get_event_loop()
    with pytest.raises(OcrUnavailableError):
        loop.run_until_complete(extract_fields("stub://test", "34a"))
