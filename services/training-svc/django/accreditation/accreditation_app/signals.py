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

"""Signal stubs for eel.training accreditation events (M42+M43)."""

import logging

logger = logging.getLogger(__name__)

# Event topic constants
MEDIA_BADGE_ISSUED = "eel.training.media_badge_issued.v1"
MEDIA_BADGE_REVOKED = "eel.training.media_badge_revoked.v1"
MEDIA_BREACH_REFERRED = "eel.training.media_breach_referred.v1"
OBSERVER_CREDENTIAL_ISSUED = "eel.training.observer_credential_issued.v1"
OBSERVER_CREDENTIAL_REVOKED = "eel.training.observer_credential_revoked.v1"
OBSERVER_DEPLOYED = "eel.training.observer_deployed.v1"
MOU_SIGNED = "eel.training.mou_signed.v1"


def emit_stub(topic: str, payload: dict) -> None:
    """Stub emitter: logs the event. Replace with real bus in production."""
    logger.info("EVENT topic=%s payload=%s", topic, payload)


def on_media_badge_issued(accreditation_id: str, badge_serial: str) -> None:
    emit_stub(
        MEDIA_BADGE_ISSUED,
        {"accreditation_id": accreditation_id, "badge_serial": badge_serial},
    )


def on_media_badge_revoked(
    accreditation_id: str, badge_serial: str, reason_code: str
) -> None:
    emit_stub(
        MEDIA_BADGE_REVOKED,
        {
            "accreditation_id": accreditation_id,
            "badge_serial": badge_serial,
            "reason_code": reason_code,
        },
    )


def on_media_breach_referred(accreditation_id: str, details: str) -> None:
    emit_stub(
        MEDIA_BREACH_REFERRED,
        {"accreditation_id": accreditation_id, "details": details},
    )


def on_observer_credential_issued(observer_id: str, credential_serial: str) -> None:
    emit_stub(
        OBSERVER_CREDENTIAL_ISSUED,
        {"observer_id": observer_id, "credential_serial": credential_serial},
    )


def on_observer_credential_revoked(
    observer_id: str, credential_serial: str, reason: str
) -> None:
    emit_stub(
        OBSERVER_CREDENTIAL_REVOKED,
        {
            "observer_id": observer_id,
            "credential_serial": credential_serial,
            "reason": reason,
        },
    )


def on_observer_deployed(observer_id: str, county_codes: list[str]) -> None:
    emit_stub(
        OBSERVER_DEPLOYED, {"observer_id": observer_id, "county_codes": county_codes}
    )


def on_mou_signed(org_id: str, signed_at: str) -> None:
    emit_stub(MOU_SIGNED, {"org_id": org_id, "signed_at": signed_at})
