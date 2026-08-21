#!/usr/bin/env python
"""Django management entry point for eel-hr (WP-5.6)."""

import os
import sys


def main() -> None:
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "hr_project.settings")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Could not import Django. Ensure it is installed and available."
        ) from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
