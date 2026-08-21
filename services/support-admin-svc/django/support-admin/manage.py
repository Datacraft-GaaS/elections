#!/usr/bin/env python
"""Django management script for support-admin (WP-5.5, M25, M27)."""

import os
import sys


def main():
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "support_admin.settings")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Could not import Django. Is it installed and DJANGO_SETTINGS_MODULE set?"
        ) from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
