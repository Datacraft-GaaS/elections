from rest_framework import serializers

from .models import ContentItem, ContentVersion, DistributionEvent, FactCheckReview


class ContentItemSerializer(serializers.ModelSerializer):
    class Meta:
        model = ContentItem
        fields = "__all__"
        read_only_fields = ["id", "created_at", "updated_at"]


class ContentVersionSerializer(serializers.ModelSerializer):
    class Meta:
        model = ContentVersion
        fields = "__all__"
        read_only_fields = ["id", "created_at"]

    def validate(self, attrs):
        item = attrs.get("content_item")
        if item and item.status == "PUBLISHED":
            raise serializers.ValidationError(
                "Cannot add versions to a published content item."
            )
        return attrs


class FactCheckReviewSerializer(serializers.ModelSerializer):
    class Meta:
        model = FactCheckReview
        fields = "__all__"
        read_only_fields = ["id"]


class DistributionEventSerializer(serializers.ModelSerializer):
    class Meta:
        model = DistributionEvent
        fields = "__all__"
        read_only_fields = ["id", "dispatched_at"]
