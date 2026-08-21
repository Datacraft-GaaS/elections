from django.apps import AppConfig


class MedicalAppConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "medical_app"
    verbose_name = "EEL Medical First Response (M60)"

    def ready(self):
        import medical_app.signals  # noqa: F401
