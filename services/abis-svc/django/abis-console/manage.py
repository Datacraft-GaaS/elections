#!/usr/bin/env python
"""Django management entry point for abis-console."""

import os
import sys


def main():
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "abis_console.settings")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Could not import Django. Is it installed and in PYTHONPATH?"
        ) from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
