from rest_framework import serializers

from .models import Budget, BudgetLine, CoBReport, PaymentVoucher, ProcurementRecord


class BudgetSerializer(serializers.ModelSerializer):
    class Meta:
        model = Budget
        fields = "__all__"
        read_only_fields = ["id", "created_at", "updated_at"]


class BudgetLineSerializer(serializers.ModelSerializer):
    class Meta:
        model = BudgetLine
        fields = "__all__"
        read_only_fields = [
            "id",
            "committed_kes",
            "spent_kes",
            "created_at",
            "updated_at",
        ]


class ProcurementRecordSerializer(serializers.ModelSerializer):
    class Meta:
        model = ProcurementRecord
        fields = "__all__"
        read_only_fields = ["id", "status", "created_at", "updated_at"]


class PaymentVoucherSerializer(serializers.ModelSerializer):
    class Meta:
        model = PaymentVoucher
        fields = "__all__"
        read_only_fields = [
            "id",
            "approved_by",
            "counter_approved_by",
            "ifmis_ref",
            "status",
            "created_at",
            "updated_at",
        ]


class VoucherApproveSerializer(serializers.Serializer):
    approver_id = serializers.CharField(max_length=36)


class VoucherCounterApproveSerializer(serializers.Serializer):
    counter_approver_id = serializers.CharField(max_length=36)


class VoucherPaySerializer(serializers.Serializer):
    ifmis_ref = serializers.CharField(max_length=100)


class CoBReportSerializer(serializers.ModelSerializer):
    class Meta:
        model = CoBReport
        fields = "__all__"
        read_only_fields = ["id", "created_at"]


class BudgetSummarySerializer(serializers.Serializer):
    budget_id = serializers.CharField()
    financial_year = serializers.CharField()
    total_kes = serializers.IntegerField()
    total_committed = serializers.IntegerField()
    total_spent = serializers.IntegerField()
    variance = serializers.IntegerField()
