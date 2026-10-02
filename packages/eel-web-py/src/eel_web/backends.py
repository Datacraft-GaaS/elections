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

"""Django auth backend delegating permissions to SpiceDB."""

from __future__ import annotations

from typing import Any

from django.contrib.auth.backends import BaseBackend
from django.contrib.auth.base_user import AbstractBaseUser
from django.http import HttpRequest


class SpiceDBBackend(BaseBackend):
    """Django authentication backend that delegates permissions to SpiceDB."""

    def has_perm(
        self,
        user_obj: AbstractBaseUser | None,
        perm: str,
        obj: Any = None,
    ) -> bool:
        """Check permission via SpiceDB. Runs synchronously (wraps async client).

        perm format: "<resource_type>:<permission>" e.g. "form34a:view"
        Stub: always returns False until SpiceDB client is configured.
        """
        return False

    def authenticate(  # type: ignore[override]
        self,
        request: HttpRequest | None,
        **kwargs: Any,
    ) -> AbstractBaseUser | None:
        # Authentication is handled by EelAuthMiddleware
        return None
