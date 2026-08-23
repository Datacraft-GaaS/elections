from django.apps import AppConfig


class EelDesignSystemConfig(AppConfig):
    name = "eel_design_system"
    label = "eel_design_system"
    verbose_name = "eel Design System"

    # No import-time hooks required; template tags are discovered by Django
    # automatically because the package ships an ``templatetags`` module.