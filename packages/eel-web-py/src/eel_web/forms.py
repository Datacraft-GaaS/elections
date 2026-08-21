"""PII-aware form fields for eel_web."""

from __future__ import annotations

from django import forms


class PIICharField(forms.CharField):
    """CharField that marks data as PII tier B in cleaned_data."""

    pii_tier: str = "B"

    def clean(self, value: str) -> str:
        value = super().clean(value)
        # In real impl: wrap in PIISensitive; here just return value
        return value


class PIIRestrictedField(PIICharField):
    """PII tier A: most restricted (e.g. biometrics, health data)."""

    pii_tier = "A"


class PIISensitiveField(PIICharField):
    """PII tier B: sensitive (e.g. name, ID number)."""

    pii_tier = "B"


class PIIOperationalField(PIICharField):
    """PII tier C: operational (e.g. IP address, device info)."""

    pii_tier = "C"
