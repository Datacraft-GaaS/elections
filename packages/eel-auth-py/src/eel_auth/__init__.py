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

from eel_auth.audit import AuditSink, LogAuditSink, NoopAuditSink
from eel_auth.jwks import AuthError, JwksCache, verify_token
from eel_auth.middleware import AuthMiddleware, require_permission
from eel_auth.spicedb import CheckResult, SpiceDBClient
from eel_auth.subject import Subject, SubjectExtractor

__all__ = [
    "JwksCache",
    "verify_token",
    "AuthError",
    "Subject",
    "SubjectExtractor",
    "SpiceDBClient",
    "CheckResult",
    "AuditSink",
    "NoopAuditSink",
    "LogAuditSink",
    "AuthMiddleware",
    "require_permission",
]
