from django.apps import AppConfig


class AccreditationAppConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "accreditation_app"
    verbose_name = "EEL Accreditation (M42+M43)"

    def ready(self):
        import accreditation_app.signals  # noqa: F401
