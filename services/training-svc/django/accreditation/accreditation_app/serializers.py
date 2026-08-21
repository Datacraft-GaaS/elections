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
        read_only_fields = ["badge_serial", "badge_issued_at", "revoked_at"]


class MediaAccreditationRevokeSerializer(serializers.Serializer):
    reason_code = serializers.ChoiceField(
        choices=["BREACH", "MISCONDUCT", "SECURITY", "EXPIRED"]
    )
    officer_id = serializers.UUIDField()
    revocation_reason = serializers.CharField(required=False, allow_blank=True)


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
