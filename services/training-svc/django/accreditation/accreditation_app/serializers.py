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
from accreditation_app.models import (
    MediaOrganisation,
    MediaAccreditation,
    ObserverOrganisation,
    IndividualObserver,
    CredentialRevocationEntry,
)


class MediaOrganisationSerializer(serializers.ModelSerializer):
    class Meta:
        model = MediaOrganisation
        fields = "__all__"


class MediaAccreditationSerializer(serializers.ModelSerializer):
    class Meta:
        model = MediaAccreditation
        fields = "__all__"
        read_only_fields = ["badge_serial", "badge_issued_at", "badge_hsm_signature"]


class MediaAccreditationRevokeSerializer(serializers.Serializer):
    """Body of `POST /media-accreditations/{id}/revoke`.

    No free-text reason field: `training_accred_badge_revocations` stores a
    `reason` column carrying `reason_code`, and the API contract
    (`packages/openapi-contracts/training-accreditation`) publishes the code
    only. A `revocation_reason` CharField used to sit here and was accepted
    into `validated_data` and then dropped by the view, so a caller could send
    an explanation and have it silently discarded -- the same shape as the
    `officer_id` drop the view's own comment records.
    """

    reason_code = serializers.ChoiceField(
        choices=["BREACH", "MISCONDUCT", "SECURITY", "EXPIRED"]
    )
    officer_id = serializers.UUIDField()


class ObserverOrganisationSerializer(serializers.ModelSerializer):
    class Meta:
        model = ObserverOrganisation
        fields = "__all__"


class IndividualObserverSerializer(serializers.ModelSerializer):
    class Meta:
        model = IndividualObserver
        fields = "__all__"
        read_only_fields = ["credential_serial", "credential_issued_at"]


class IndividualObserverRevokeSerializer(serializers.Serializer):
    reason = serializers.CharField()
    officer_id = serializers.UUIDField()


class CredentialRevocationEntrySerializer(serializers.ModelSerializer):
    class Meta:
        model = CredentialRevocationEntry
        fields = "__all__"
