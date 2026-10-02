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
