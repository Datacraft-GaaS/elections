"""SSML and plain-text menu builder for IVR responses."""

from __future__ import annotations

_LANG_CODES: dict[str, str] = {
    "en": "en-KE",
    "sw": "sw-KE",
    "ki": "ki-KE",
    "luo": "luo-KE",
}


def build_ssml(text: str, lang: str = "en") -> str:
    """Wrap text in SSML speak tags with the appropriate language code.

    Args:
            text: Plain text to speak.
            lang: BCP-47 short code (en, sw, ki, luo). Defaults to en.

    Returns:
            SSML string suitable for Africa's Talking TTS.
    """
    assert text, "text required"
    lang_code = _LANG_CODES.get(lang, "en-KE")
    escaped = text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return f'<speak xml:lang="{lang_code}">{escaped}</speak>'


def build_menu(options: list[tuple[str, str]], lang: str = "en") -> str:
    """Build a numbered DTMF menu as SSML.

    Args:
            options: List of (digit, description) tuples.
            lang: Language code for SSML.

    Returns:
            SSML string for the full menu.
    """
    assert options, "options required"
    lines = " ".join(f"Press {digit} for {desc}." for digit, desc in options)
    return build_ssml(lines, lang)
