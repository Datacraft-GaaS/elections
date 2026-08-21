from django.apps import AppConfig


class LmsAppConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "lms_app"
    verbose_name = "Poll Worker Training LMS"

    def ready(self):
        import lms_app.signals  # noqa: F401
