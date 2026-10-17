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

from rest_framework import viewsets, status
from rest_framework.decorators import action, api_view
from rest_framework.request import Request
from rest_framework.response import Response

from accreditation_app.models import (
    BadgeRevocationEntry,
    CredentialRevocationEntry,
    IndividualObserver,
    MediaAccreditation,
    MediaOrganisation,
    ObserverOrganisation,
)
from accreditation_app.serializers import (
    MediaOrganisationSerializer,
    MediaAccreditationSerializer,
    MediaAccreditationRevokeSerializer,
    ObserverOrganisationSerializer,
    IndividualObserverSerializer,
    IndividualObserverRevokeSerializer,
)
from accreditation_app.badge import verify_badge
from accreditation_app.crl import generate_crl
from accreditation_app import signals as sig

logger = logging.getLogger(__name__)


class MediaOrganisationViewSet(viewsets.ModelViewSet):
    queryset = MediaOrganisation.objects.all()
    serializer_class = MediaOrganisationSerializer


class MediaAccreditationViewSet(viewsets.ModelViewSet):
    queryset = MediaAccreditation.objects.select_related("organisation").all()
    serializer_class = MediaAccreditationSerializer

    @action(detail=True, methods=["post"], url_path="revoke")
    def revoke(self, request: Request, pk=None) -> Response:
        accreditation = self.get_object()
        ser = MediaAccreditationRevokeSerializer(data=request.data)
        ser.is_valid(raise_exception=True)

        reason_code: str = ser.validated_data["reason_code"]
        officer_id = str(ser.validated_data["officer_id"])

        if accreditation.status == "revoked":
            return Response(
                {"detail": "Already revoked."}, status=status.HTTP_400_BAD_REQUEST
            )

        badge_serial = accreditation.badge_serial
        accreditation.status = "revoked"
        accreditation.save()

        # Writing the revocation row is what actually revokes the badge:
        # `crl.generate_crl()` builds the certificate revocation list from
        # BadgeRevocationEntry, so a revocation that stops at `status` leaves
        # the serial verifying as valid. The view previously set the status and
        # dropped both `officer_id` and this insert.
        if badge_serial:
            BadgeRevocationEntry.objects.create(
                accreditation=accreditation,
                badge_serial=str(badge_serial),
                officer_id=officer_id,
                reason=reason_code,
            )
            sig.on_media_badge_revoked(
                str(accreditation.id), str(badge_serial), reason_code
            )

        return Response(
            {
                "status": "REVOKED",
                "badge_serial": str(badge_serial) if badge_serial else None,
            }
        )


class ObserverOrganisationViewSet(viewsets.ModelViewSet):
    queryset = ObserverOrganisation.objects.all()
    serializer_class = ObserverOrganisationSerializer


class IndividualObserverViewSet(viewsets.ModelViewSet):
    queryset = IndividualObserver.objects.select_related("organisation").all()
    serializer_class = IndividualObserverSerializer

    @action(detail=True, methods=["post"], url_path="revoke")
    def revoke(self, request: Request, pk=None) -> Response:
        observer = self.get_object()
        ser = IndividualObserverRevokeSerializer(data=request.data)
        ser.is_valid(raise_exception=True)

        reason: str = ser.validated_data["reason"]
        officer_id = str(ser.validated_data["officer_id"])

        if observer.status == "credential_revoked":
            return Response(
                {"detail": "Already revoked."}, status=status.HTTP_400_BAD_REQUEST
            )

        credential_serial = observer.credential_serial
        observer.status = "credential_revoked"
        observer.save()

        if credential_serial:
            # `observer` is required: the table declares `observer_id NOT NULL`,
            # and the model omits it, so this insert raised against the real
            # schema. SQLite builds its tables from the model and never noticed.
            CredentialRevocationEntry.objects.create(
                observer=observer,
                credential_serial=str(credential_serial),
                reason=reason,
                officer_id=officer_id,
            )
            sig.on_observer_credential_revoked(
                str(observer.id), str(credential_serial), reason
            )

        return Response({"status": "REVOKED"})


@api_view(["GET"])
def verify_badge_view(request: Request, serial: str) -> Response:
    """Public view: check badge status against CRL. Returns no PII."""
    crl = generate_crl()
    badge_status = verify_badge(serial, crl)
    return Response({"serial": serial, "status": badge_status})


@api_view(["GET"])
def crl_view(request: Request) -> Response:
    """Return current CRL JSON."""
    return Response(generate_crl())
