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
