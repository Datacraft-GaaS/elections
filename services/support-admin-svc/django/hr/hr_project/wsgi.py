"""WSGI config for eel-hr (WP-5.6)."""

import os

from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "hr_project.settings")

application = get_wsgi_application()
