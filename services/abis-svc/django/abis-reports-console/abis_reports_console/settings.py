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

"""Django settings for abis-reports-console.

SQLite for test (DJANGO_TEST_SQLITE=true); PostgreSQL via DATABASE_URL for production.
"""

import os

import dj_database_url

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

SECRET_KEY = os.environ.get("DJANGO_SECRET_KEY", "dev-insecure-key-change-in-prod")

DEBUG = os.environ.get("DJANGO_DEBUG", "true").lower() == "true"

ALLOWED_HOSTS = os.environ.get("DJANGO_ALLOWED_HOSTS", "*").split(",")

INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    "django_htmx",
    # Required by ArrayField. Several columns here are TEXT[] in the
    # migrations (access_zones, audience, renderer_pipeline, ...), and Django
    # refuses to load an ArrayField without this app installed.
    "django.contrib.postgres",
    "widget_tweaks",
    "abis_reports_console.apps.reports",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
    "django_htmx.middleware.HtmxMiddleware",
    "abis_reports_console.middleware.spicedb.SpiceDBPermissionMiddleware",
]

ROOT_URLCONF = "abis_reports_console.urls"

TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [os.path.join(BASE_DIR, "templates")],
        "APP_DIRS": True,
        "OPTIONS": {
            "context_processors": [
                "django.template.context_processors.debug",
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        },
    },
]

WSGI_APPLICATION = "abis_reports_console.wsgi.application"
ASGI_APPLICATION = "abis_reports_console.asgi.application"

# Database: SQLite for tests, PostgreSQL for production.
_use_sqlite = os.environ.get("DJANGO_TEST_SQLITE", "false").lower() == "true"

if _use_sqlite:
    DATABASES = {
        "default": {
            "ENGINE": "django.db.backends.sqlite3",
            "NAME": os.path.join(BASE_DIR, "db.sqlite3"),
        }
    }
else:
    _db_url = os.environ.get(
        "DATABASE_URL", "postgresql://eel:eel@localhost:5432/eel_abis"
    )
    DATABASES = {"default": dj_database_url.parse(_db_url, conn_max_age=600)}

AUTH_PASSWORD_VALIDATORS = [
    {
        "NAME": "django.contrib.auth.password_validation.UserAttributeSimilarityValidator"
    },
    {"NAME": "django.contrib.auth.password_validation.MinimumLengthValidator"},
    {"NAME": "django.contrib.auth.password_validation.CommonPasswordValidator"},
    {"NAME": "django.contrib.auth.password_validation.NumericPasswordValidator"},
]

LANGUAGE_CODE = "en-gb"
TIME_ZONE = "Africa/Nairobi"
USE_I18N = True
USE_TZ = True

STATIC_URL = "/static/"
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"

# MinIO / S3 storage for report artifacts.
MINIO_ENDPOINT = os.environ.get("MINIO_ENDPOINT", "http://localhost:9000")
MINIO_BUCKET = os.environ.get("MINIO_BUCKET", "eel-reports")
AWS_ACCESS_KEY_ID = os.environ.get("MINIO_ACCESS_KEY", "minioadmin")
AWS_SECRET_ACCESS_KEY = os.environ.get("MINIO_SECRET_KEY", "minioadmin")
AWS_S3_ENDPOINT_URL = MINIO_ENDPOINT
AWS_STORAGE_BUCKET_NAME = MINIO_BUCKET
AWS_S3_ADDRESSING_STYLE = "path"
# Signed URL expiry for artifact downloads (seconds).
REPORT_SIGNED_URL_EXPIRY = int(os.environ.get("REPORT_SIGNED_URL_EXPIRY", "300"))

# SpiceDB permission check.
# Set SPICEDB_ENDPOINT + SPICEDB_TOKEN in production. Leave unset to disable (dev/CI).
SPICEDB_ENDPOINT = os.environ.get("SPICEDB_ENDPOINT", "")
SPICEDB_TOKEN = os.environ.get("SPICEDB_TOKEN", "")
