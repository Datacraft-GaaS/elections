from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import viewsets
from rest_framework.filters import OrderingFilter

from .models import ContentItem, ContentVersion, DistributionEvent, FactCheckReview
from .serializers import (
    ContentItemSerializer,
    ContentVersionSerializer,
    DistributionEventSerializer,
    FactCheckReviewSerializer,
)


class ContentItemViewSet(viewsets.ModelViewSet):
    queryset = ContentItem.objects.all()
    serializer_class = ContentItemSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = [
        "status",
        "language",
        "format",
        "topic_category",
        "election_cycle_tag",
    ]
    ordering_fields = ["created_at", "updated_at", "expiry_date"]


class ContentVersionViewSet(viewsets.ModelViewSet):
    serializer_class = ContentVersionSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["content_item"]

    def get_queryset(self):
        qs = ContentVersion.objects.all()
        item_id = self.request.query_params.get("content_item")
        if item_id:
            qs = qs.filter(content_item_id=item_id)
        return qs

    def perform_create(self, serializer):
        item = serializer.validated_data["content_item"]
        if item.status == "PUBLISHED":
            from rest_framework.exceptions import ValidationError

            raise ValidationError("Cannot add versions to a published content item.")
        serializer.save()


class FactCheckReviewViewSet(viewsets.ModelViewSet):
    queryset = FactCheckReview.objects.all()
    serializer_class = FactCheckReviewSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["content_item", "decision"]


class DistributionEventViewSet(viewsets.ModelViewSet):
    queryset = DistributionEvent.objects.all()
    serializer_class = DistributionEventSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["content_item", "channel", "county_code"]
