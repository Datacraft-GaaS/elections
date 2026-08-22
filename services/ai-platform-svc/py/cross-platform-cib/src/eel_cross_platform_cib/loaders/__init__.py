"""Platform-specific data loaders with schema unification into Post."""

from .facebook import FacebookLoader
from .telegram import TelegramLoader
from .tiktok import TikTokLoader
from .twitter import TwitterLoader
from .whatsapp_broadcast import WhatsAppBroadcastLoader

__all__ = [
    "FacebookLoader",
    "TelegramLoader",
    "TikTokLoader",
    "TwitterLoader",
    "WhatsAppBroadcastLoader",
]
