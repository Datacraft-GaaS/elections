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

"""RFC 8785 canonical JSON serialization (pure Python approximation)."""

from __future__ import annotations

import json
import math
from typing import Any


def canonicalize(obj: Any) -> bytes:
    """Produce deterministic JSON bytes per RFC 8785 (approximate).

    Differences from the full spec:
    - Float repr uses Python's repr() rather than true IEEE 754 shortest.
    - String escaping follows json.dumps defaults.
    """
    return _serialize(obj).encode("utf-8")


def _serialize(obj: Any) -> str:
    if obj is None:
        return "null"
    if isinstance(obj, bool):
        return "true" if obj else "false"
    if isinstance(obj, int):
        return str(obj)
    if isinstance(obj, float):
        if math.isnan(obj) or math.isinf(obj):
            raise ValueError(f"RFC 8785 does not allow NaN/Infinity: {obj}")
        return repr(obj)
    if isinstance(obj, str):
        return json.dumps(obj, ensure_ascii=False)
    if isinstance(obj, (list, tuple)):
        items = ",".join(_serialize(v) for v in obj)
        return f"[{items}]"
    if isinstance(obj, dict):
        # Sort by unicode code point order of key strings
        pairs = sorted(obj.items(), key=lambda kv: kv[0])
        parts = ",".join(
            f"{json.dumps(k, ensure_ascii=False)}:{_serialize(v)}" for k, v in pairs
        )
        return "{" + parts + "}"
    raise TypeError(f"Unsupported type for canonicalization: {type(obj)}")
