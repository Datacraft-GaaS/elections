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
