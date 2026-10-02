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
