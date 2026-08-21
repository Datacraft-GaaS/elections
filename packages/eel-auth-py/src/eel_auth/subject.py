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
