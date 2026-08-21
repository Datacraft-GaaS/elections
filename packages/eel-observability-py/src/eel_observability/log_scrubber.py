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
