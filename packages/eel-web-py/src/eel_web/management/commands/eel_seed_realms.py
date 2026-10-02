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

"""Management command: import Keycloak realm JSON to a dev Keycloak instance."""

from __future__ import annotations

import argparse

from django.core.management.base import BaseCommand


class Command(BaseCommand):
    help = "Import Keycloak realm JSON to a dev Keycloak instance."

    def add_arguments(self, parser: argparse.ArgumentParser) -> None:
        parser.add_argument("realm_json", help="Path to realm JSON file")
        parser.add_argument("--keycloak-url", default="http://localhost:8080")
        parser.add_argument("--admin-user", default="admin")
        parser.add_argument("--admin-password", default="admin")

    def handle(self, *args: object, **options: object) -> None:
        realm_json = str(options["realm_json"])
        self.stdout.write(f"Seeding realm from {realm_json}...")
        # Stub: in real impl, POST to /admin/realms
        self.stdout.write("Done (stub - configure Keycloak URL and credentials)")
