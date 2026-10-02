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

import logging

from django.conf import settings
from django.db.models.signals import post_save
from django.dispatch import receiver

from .models import ContentItem, ContentStatus

_log_ = logging.getLogger(__name__)

PUBLISH_EVENTS = {
    ContentStatus.PUBLISHED: "eel.training.content_published.v1",
    ContentStatus.EXPIRED: "eel.training.content_expired.v1",
    ContentStatus.ARCHIVED: "eel.training.content_archived.v1",
}


def _emit_kafka_event(topic: str, payload: dict) -> None:
    if getattr(settings, "USE_KAFKA", False):
        try:
            from kafka import KafkaProducer
            import json

            producer = KafkaProducer(
                bootstrap_servers=settings.KAFKA_BOOTSTRAP_SERVERS,
                value_serializer=lambda v: json.dumps(v).encode("utf-8"),
            )
            producer.send(topic, payload)
            producer.flush()
            _log_.info("kafka event sent: topic=%s payload=%s", topic, payload)
        except Exception as exc:
            _log_.error("kafka emit failed: topic=%s error=%s", topic, exc)
    else:
        _log_.info("[kafka-stub] topic=%s payload=%s", topic, payload)


@receiver(post_save, sender=ContentItem)
def on_content_item_save(
    sender, instance: ContentItem, created: bool, **kwargs
) -> None:
    topic = PUBLISH_EVENTS.get(instance.status)
    if topic:
        payload = {
            "event": topic,
            "content_item_id": str(instance.id),
            "status": instance.status,
            "title": instance.title,
        }
        _emit_kafka_event(topic, payload)
