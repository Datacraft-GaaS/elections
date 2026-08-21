import logging
from datetime import datetime, timezone

from django.utils import timezone as djtimezone
from rest_framework import viewsets, status
from rest_framework.decorators import action, api_view
from rest_framework.request import Request
from rest_framework.response import Response

from accreditation_app.models import (
    MediaOrganisation,
    MediaAccreditation,
    ObserverOrganisation,
    IndividualObserver,
    CredentialRevocationEntry,
)
from accreditation_app.serializers import (
    MediaOrganisationSerializer,
    MediaAccreditationSerializer,
    MediaAccreditationRevokeSerializer,
    ObserverOrganisationSerializer,
    IndividualObserverSerializer,
    IndividualObserverRevokeSerializer,
)
from accreditation_app.badge import generate_badge_serial, verify_badge
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
        revocation_reason: str = ser.validated_data.get("revocation_reason", "")

        if accreditation.status == "REVOKED":
            return Response(
                {"detail": "Already revoked."}, status=status.HTTP_400_BAD_REQUEST
            )

        badge_serial = accreditation.badge_serial
        accreditation.status = "REVOKED"
        accreditation.revocation_reason = revocation_reason
        accreditation.revoked_at = djtimezone.now()
        accreditation.save()

        if badge_serial:
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

        if observer.status == "REVOKED":
            return Response(
                {"detail": "Already revoked."}, status=status.HTTP_400_BAD_REQUEST
            )

        credential_serial = observer.credential_serial
        observer.status = "REVOKED"
        observer.revocation_reason = reason
        observer.save()

        if credential_serial:
            CredentialRevocationEntry.objects.create(
                credential_serial=credential_serial,
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
