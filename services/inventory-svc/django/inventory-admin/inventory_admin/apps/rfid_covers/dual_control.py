"""Dual-control rule for tamper-alert resolution (wave-2 HM1).

Pure logic, no Django imports, so it is testable without settings/database.
Two distinct officers must approve a tamper-alert resolution: the acting
resolver and a countersigning second officer. Self-approval is rejected.
"""

from __future__ import annotations


class DualApprovalError(ValueError):
    """Raised when second-officer approval is missing or degenerate."""


def validate_dual_approval(resolver: str, second_officer: str | None) -> str:
    """Return the cleaned second-officer id, enforcing the two-person rule.

    Raises DualApprovalError when:
      - the second officer id is missing/blank;
      - it equals the resolver (self-approval).
    """
    second = (second_officer or "").strip()
    if not second:
        raise DualApprovalError(
            "second officer approval is required to resolve a tamper alert"
        )
    if second == (resolver or "").strip():
        raise DualApprovalError("second officer must differ from the resolving officer")
    return second


def countersigned_note(resolver: str, second_officer: str, note: str) -> str:
    """Compose the durable audit note carrying both officers."""
    base = (note or "").strip()
    audit = f"[resolved by {resolver}; countersigned by {second_officer}]"
    return f"{base} {audit}" if base else audit
