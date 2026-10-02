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
