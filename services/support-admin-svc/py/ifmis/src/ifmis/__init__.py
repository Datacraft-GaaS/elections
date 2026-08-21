"""IFMIS adapter client for support-admin-svc (WP-5.6)."""

from .client import IfmisClient
from .models import GlPostingRequest, GlPostingResponse, IfmisVoucher

__all__ = ["IfmisClient", "GlPostingRequest", "GlPostingResponse", "IfmisVoucher"]
