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

from django.utils import timezone
from rest_framework import viewsets, status
from rest_framework.decorators import action
from rest_framework.request import Request
from rest_framework.response import Response

from medical_app.models import (
    MedicalIncident,
    DispatchRequest,
    ClusterAlert,
    MedicalUnit,
)
from medical_app.serializers import (
    MedicalIncidentSerializer,
    DispatchRequestSerializer,
    DispatchActionSerializer,
    ClusterAlertSerializer,
    MedicalUnitSerializer,
)
from medical_app import dispatch as dispatch_mod
from medical_app import signals as sig

logger = logging.getLogger(__name__)


class MedicalIncidentViewSet(viewsets.ModelViewSet):
    queryset = MedicalIncident.objects.all()
    serializer_class = MedicalIncidentSerializer

    def perform_create(self, serializer):
        incident = serializer.save()
        sig.on_incident_reported(
            str(incident.id), incident.station_id, incident.severity
        )

    @action(detail=True, methods=["post"], url_path="dispatch")
    def dispatch(self, request: Request, pk=None) -> Response:
        incident = self.get_object()
        ser = DispatchActionSerializer(data=request.data)
        ser.is_valid(raise_exception=True)

        provider: str = ser.validated_data["dispatched_to"]
        eta: int | None = ser.validated_data.get("eta_minutes")

        # Idempotency: only one dispatch per incident per provider
        existing = DispatchRequest.objects.filter(
            incident=incident, dispatched_to=provider
        ).first()
        if existing:
            return Response(
                DispatchRequestSerializer(existing).data,
                status=status.HTTP_200_OK,
            )

        dispatch_result = dispatch_mod.dispatch(provider, str(incident.id), eta)

        dispatch_obj = DispatchRequest.objects.create(
            incident=incident,
            dispatched_to=provider,
            eta_minutes=eta,
        )
        incident.status = "dispatched"
        incident.save()

        sig.on_dispatch_requested(str(dispatch_obj.id), str(incident.id), provider)

        return Response(
            {
                **DispatchRequestSerializer(dispatch_obj).data,
                "dispatch_ack": dispatch_result,
            },
            status=status.HTTP_201_CREATED,
        )

    @action(detail=True, methods=["post"], url_path="resolve")
    def resolve(self, request: Request, pk=None) -> Response:
        incident = self.get_object()
        if incident.status == "resolved":
            return Response(
                {"detail": "Already resolved."}, status=status.HTTP_400_BAD_REQUEST
            )
        incident.status = "resolved"
        incident.save()
        return Response({"status": "resolved"})


class DispatchRequestViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = DispatchRequest.objects.select_related("incident").all()
    serializer_class = DispatchRequestSerializer


class ClusterAlertViewSet(viewsets.ModelViewSet):
    queryset = ClusterAlert.objects.all()
    serializer_class = ClusterAlertSerializer

    def perform_create(self, serializer):
        alert = serializer.save()
        if alert.incident_count >= alert.alert_threshold:
            sig.on_cluster_alert(str(alert.id), alert.county_code, alert.incident_count)

    @action(detail=True, methods=["post"], url_path="resolve")
    def resolve(self, request: Request, pk=None) -> Response:
        alert = self.get_object()
        if alert.resolved_at is not None:
            return Response(
                {"detail": "Already resolved."}, status=status.HTTP_400_BAD_REQUEST
            )
        alert.resolved_at = timezone.now()
        alert.save()
        return Response({"status": "resolved", "resolved_at": alert.resolved_at})


class MedicalUnitViewSet(viewsets.ModelViewSet):
    queryset = MedicalUnit.objects.all()
    serializer_class = MedicalUnitSerializer
