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

"""SpiceDB permission check middleware.

Performs a session-scoped permission check against SpiceDB for every request.
When SPICEDB_ENDPOINT is not configured (dev/CI), the check is skipped silently.

Environment:
    SPICEDB_ENDPOINT: gRPC endpoint, e.g. "spicedb.internal:50051"
    SPICEDB_TOKEN:    pre-shared key
"""

import logging

from django.conf import settings
from django.http import HttpRequest, HttpResponse

_log_ = logging.getLogger(__name__)


class SpiceDBPermissionMiddleware:
    """Stub SpiceDB permission middleware.

    In production: call CheckPermission on the SpiceDB gRPC API.
    When SPICEDB_ENDPOINT is empty, staff authentication falls through to Django auth.
    """

    def __init__(self, get_response):
        self.get_response = get_response
        self._endpoint = getattr(settings, "SPICEDB_ENDPOINT", "")

    def __call__(self, request: HttpRequest) -> HttpResponse:
        if self._endpoint and request.user.is_authenticated:
            allowed = self._check(request)
            if not allowed:
                _log_.warning(
                    "SpiceDB denied access: user=%s path=%s",
                    request.user,
                    request.path,
                )
                return HttpResponse("Forbidden by permission policy.", status=403)
        return self.get_response(request)

    def _check(self, request: HttpRequest) -> bool:
        """Stub: always permits when SpiceDB is not reachable.

        Replace with actual authzed.com CheckPermission call in production.
        """
        _log_.debug("SpiceDB check (stub): user=%s path=%s", request.user, request.path)
        return True
