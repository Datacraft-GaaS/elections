from django.apps import AppConfig


class EelWebConfig(AppConfig):
    name = "eel_web"
    default_auto_field = "django.db.models.BigAutoField"  # type: ignore[assignment]
    verbose_name = "Eel Web"

    def ready(self) -> None:
        import eel_web.signals  # noqa: F401
