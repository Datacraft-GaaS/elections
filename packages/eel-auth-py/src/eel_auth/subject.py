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

from dataclasses import dataclass, field
from typing import Any


@dataclass
class Subject:
    sub: str
    username: str | None
    email: str | None
    roles: list[str]
    realm: str
    raw_claims: dict[str, Any] = field(default_factory=dict)


class SubjectExtractor:
    """Extracts a Subject from validated JWT claims."""

    def __init__(self, realm: str, roles_claim: str = "realm_access.roles") -> None:
        self._realm = realm
        self._roles_claim = roles_claim

    def _resolve_claim(self, claims: dict[str, Any], path: str) -> Any:
        """Traverse dotted path in nested dict."""
        parts = path.split(".")
        node: Any = claims
        for part in parts:
            if not isinstance(node, dict):
                return None
            node = node.get(part)
        return node

    def extract(self, claims: dict[str, Any]) -> Subject:
        sub = str(claims.get("sub", ""))
        username = claims.get("preferred_username") or claims.get("username")
        email = claims.get("email")
        roles_raw = self._resolve_claim(claims, self._roles_claim)
        roles: list[str] = list(roles_raw) if isinstance(roles_raw, list) else []
        return Subject(
            sub=sub,
            username=username,
            email=email,
            roles=roles,
            realm=self._realm,
            raw_claims=claims,
        )
