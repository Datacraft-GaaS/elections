from django.apps import AppConfig


class EelDesignSystemConfig(AppConfig):
    name = "eel_design_system"
    label = "eel_design_system"
    verbose_name = "eel Design System"

    def ready(self) -> None:
        pass
