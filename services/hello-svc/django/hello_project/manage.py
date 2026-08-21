#!/usr/bin/env python
"""Django management entry point for eel-hello-django."""

import os
import sys


def main() -> None:
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "hello_project.settings")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError("Could not import Django.") from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
