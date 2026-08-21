from django.db.models import Sum
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import status, viewsets
from rest_framework.decorators import action
from rest_framework.filters import OrderingFilter
from rest_framework.request import Request
from rest_framework.response import Response

from .models import Budget, BudgetLine, CoBReport, PaymentVoucher, ProcurementRecord
from .serializers import (
    BudgetLineSerializer,
    BudgetSerializer,
    BudgetSummarySerializer,
    CoBReportSerializer,
    PaymentVoucherSerializer,
    ProcurementRecordSerializer,
    VoucherApproveSerializer,
    VoucherCounterApproveSerializer,
    VoucherPaySerializer,
)


class BudgetViewSet(viewsets.ModelViewSet):
    queryset = Budget.objects.all()
    serializer_class = BudgetSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["status", "financial_year", "election_id"]

    @action(detail=True, methods=["post"], url_path="approve")
    def approve(self, request: Request, pk: str | None = None) -> Response:
        budget = self.get_object()
        approver_id = request.data.get("approved_by")
        if not approver_id:
            return Response(
                {"detail": "approved_by is required"},
                status=status.HTTP_400_BAD_REQUEST,
            )
        budget.approved_by = approver_id
        budget.status = (
            Budget.BudgetStatus.APPROVED
            if hasattr(Budget, "BudgetStatus")
            else "approved"
        )
        budget.save(update_fields=["approved_by", "status", "updated_at"])
        return Response(BudgetSerializer(budget).data)

    @action(detail=True, methods=["get"], url_path="summary")
    def summary(self, request: Request, pk: str | None = None) -> Response:
        budget = self.get_object()
        agg = budget.lines.aggregate(
            total_committed=Sum("committed_kes"),
            total_spent=Sum("spent_kes"),
        )
        data = {
            "budget_id": budget.id,
            "financial_year": budget.financial_year,
            "total_kes": budget.total_kes,
            "total_committed": agg["total_committed"] or 0,
            "total_spent": agg["total_spent"] or 0,
            "variance": budget.total_kes - (agg["total_spent"] or 0),
        }
        return Response(BudgetSummarySerializer(data).data)


class BudgetLineViewSet(viewsets.ModelViewSet):
    queryset = BudgetLine.objects.all()
    serializer_class = BudgetLineSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["budget", "programme_code"]


class ProcurementRecordViewSet(viewsets.ModelViewSet):
    queryset = ProcurementRecord.objects.all()
    serializer_class = ProcurementRecordSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["budget_line", "vendor_id", "status", "procurement_method"]


class PaymentVoucherViewSet(viewsets.ModelViewSet):
    queryset = PaymentVoucher.objects.all()
    serializer_class = PaymentVoucherSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["procurement", "status"]

    @action(detail=True, methods=["post"], url_path="approve")
    def approve(self, request: Request, pk: str | None = None) -> Response:
        voucher = self.get_object()
        ser = VoucherApproveSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        try:
            voucher.approve(ser.validated_data["approver_id"])
        except ValueError as exc:
            return Response(
                {"detail": str(exc)}, status=status.HTTP_422_UNPROCESSABLE_ENTITY
            )
        return Response(PaymentVoucherSerializer(voucher).data)

    @action(detail=True, methods=["post"], url_path="counter-approve")
    def counter_approve(self, request: Request, pk: str | None = None) -> Response:
        voucher = self.get_object()
        ser = VoucherCounterApproveSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        try:
            voucher.counter_approve(ser.validated_data["counter_approver_id"])
        except ValueError as exc:
            return Response(
                {"detail": str(exc)}, status=status.HTTP_422_UNPROCESSABLE_ENTITY
            )
        return Response(PaymentVoucherSerializer(voucher).data)

    @action(detail=True, methods=["post"], url_path="pay")
    def pay(self, request: Request, pk: str | None = None) -> Response:
        voucher = self.get_object()
        ser = VoucherPaySerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        try:
            voucher.mark_paid(ser.validated_data["ifmis_ref"])
        except ValueError as exc:
            return Response(
                {"detail": str(exc)}, status=status.HTTP_422_UNPROCESSABLE_ENTITY
            )
        return Response(PaymentVoucherSerializer(voucher).data)


class CoBReportViewSet(viewsets.ModelViewSet):
    queryset = CoBReport.objects.all()
    serializer_class = CoBReportSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["budget", "reporting_period"]

    @action(detail=False, methods=["get"], url_path="export")
    def export(self, request: Request) -> Response:
        qs = self.filter_queryset(self.get_queryset())
        data = CoBReportSerializer(qs, many=True).data
        return Response({"reports": data, "count": len(data)})
