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

from __future__ import annotations

import logging
import re

# Patterns to scrub
_SCRUB_PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"\b\d{8,12}\b"), "[ID_REDACTED]"),  # national IDs
    (
        re.compile(r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b", re.I),
        "[EMAIL_REDACTED]",
    ),
    (re.compile(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b"), "[CARD_REDACTED]"),
]


def _scrub(text: str) -> str:
    for pattern, replacement in _SCRUB_PATTERNS:
        text = pattern.sub(replacement, text)
    return text


class PiiScrubberFilter(logging.Filter):
    """Logging filter that scrubs PII patterns from log records."""

    def filter(self, record: logging.LogRecord) -> bool:
        record.msg = _scrub(str(record.msg))
        if record.args:
            if isinstance(record.args, dict):
                record.args = {k: _scrub(str(v)) for k, v in record.args.items()}
            elif isinstance(record.args, tuple):
                record.args = tuple(_scrub(str(a)) for a in record.args)
        return True


def install_scrubber(logger: logging.Logger | None = None) -> None:
    """Install PiiScrubberFilter on the given logger (or root logger)."""
    target = logger if logger is not None else logging.getLogger()
    target.addFilter(PiiScrubberFilter())
