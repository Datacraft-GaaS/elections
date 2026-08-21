import logging

from django.conf import settings
from django.db.models.signals import post_save
from django.dispatch import receiver

from .models import WorkerCertificate, WorkerEnrollment, EnrollmentStatus

_log_ = logging.getLogger(__name__)


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


@receiver(post_save, sender=WorkerCertificate)
def on_certificate_save(
    sender, instance: WorkerCertificate, created: bool, **kwargs
) -> None:
    if created:
        _emit_kafka_event(
            "eel.training.worker_certified.v1",
            {
                "event": "eel.training.worker_certified.v1",
                "certificate_id": str(instance.id),
                "worker_id": str(instance.worker_id),
                "station_code": instance.station_code,
            },
        )
    elif instance.revoked:
        _emit_kafka_event(
            "eel.training.worker_certification_revoked.v1",
            {
                "event": "eel.training.worker_certification_revoked.v1",
                "certificate_id": str(instance.id),
                "worker_id": str(instance.worker_id),
                "revoked_at": str(instance.revoked_at),
            },
        )


@receiver(post_save, sender=WorkerEnrollment)
def on_enrollment_save(
    sender, instance: WorkerEnrollment, created: bool, **kwargs
) -> None:
    if (
        instance.status == EnrollmentStatus.PASSED
        and instance.module.module_type == "REFRESHER"
    ):
        _emit_kafka_event(
            "eel.training.refresher_completed.v1",
            {
                "event": "eel.training.refresher_completed.v1",
                "worker_id": str(instance.worker_id),
                "module_id": str(instance.module_id),
            },
        )
    elif (
        instance.status == EnrollmentStatus.PASSED
        and instance.module.module_type == "BRIEFING"
    ):
        _emit_kafka_event(
            "eel.training.briefing_confirmed.v1",
            {
                "event": "eel.training.briefing_confirmed.v1",
                "worker_id": str(instance.worker_id),
                "module_id": str(instance.module_id),
            },
        )
    if (
        instance.attempts >= instance.module.max_retakes
        and instance.status == EnrollmentStatus.FAILED
    ):
        _emit_kafka_event(
            "eel.training.third_fail_triggered.v1",
            {
                "event": "eel.training.third_fail_triggered.v1",
                "worker_id": str(instance.worker_id),
                "module_id": str(instance.module_id),
                "attempts": instance.attempts,
            },
        )
