from rest_framework import serializers
from medical_app.models import (
    MedicalIncident,
    DispatchRequest,
    ClusterAlert,
    MedicalUnit,
)
from medical_app.no_phi import validate_no_phi


class MedicalIncidentSerializer(serializers.ModelSerializer):
    class Meta:
        model = MedicalIncident
        fields = "__all__"
        read_only_fields = ["reported_at"]

    def validate(self, data):
        validate_no_phi(data)
        return data


class DispatchRequestSerializer(serializers.ModelSerializer):
    class Meta:
        model = DispatchRequest
        fields = "__all__"
        read_only_fields = ["dispatched_at"]


class DispatchActionSerializer(serializers.Serializer):
    dispatched_to = serializers.ChoiceField(choices=["KRCS", "MoH", "NDMU"])
    eta_minutes = serializers.IntegerField(required=False, allow_null=True)


class ClusterAlertSerializer(serializers.ModelSerializer):
    class Meta:
        model = ClusterAlert
        fields = "__all__"
        read_only_fields = ["raised_at"]


class MedicalUnitSerializer(serializers.ModelSerializer):
    class Meta:
        model = MedicalUnit
        fields = "__all__"
