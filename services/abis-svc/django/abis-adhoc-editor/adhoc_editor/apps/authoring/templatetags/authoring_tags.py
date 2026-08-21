from django import template

register = template.Library()


@register.filter(name="lookup")
def lookup(d: dict, key: str) -> str:
    """Template filter: {{ row|lookup:header }}."""
    if isinstance(d, dict):
        val = d.get(key, "")
        return str(val) if val is not None else ""
    return ""
