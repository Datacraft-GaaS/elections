"""Context processors that inject design token variables into Django templates."""

from __future__ import annotations

import json
from pathlib import Path

_TOKENS_PATH = Path(__file__).parent.parent.parent / "tokens.json"


def _load_tokens() -> dict:
    with open(_TOKENS_PATH) as f:
        return json.load(f)


_TOKENS: dict | None = None


def design_tokens(request) -> dict:  # noqa: ANN001
    """Inject design tokens into every template context."""
    global _TOKENS
    if _TOKENS is None:
        _TOKENS = _load_tokens()
    return {"eel_tokens": _TOKENS}
