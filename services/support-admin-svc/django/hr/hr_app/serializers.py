"""DRF serializers for M55 HR-Payroll (WP-5.6)."""

from rest_framework import serializers

from .models import (
    JobApplication,
    PayrollRun,
    PayslipLine,
    StaffContract,
    StatutoryDeduction,
)


class JobApplicationSerializer(serializers.ModelSerializer):
    class Meta:
        model = JobApplication
        fields = "__all__"


class StaffContractSerializer(serializers.ModelSerializer):
    class Meta:
        model = StaffContract
        fields = "__all__"


class PayrollRunSerializer(serializers.ModelSerializer):
    class Meta:
        model = PayrollRun
        fields = "__all__"


class PayslipLineSerializer(serializers.ModelSerializer):
    class Meta:
        model = PayslipLine
        fields = "__all__"


class StatutoryDeductionSerializer(serializers.ModelSerializer):
    class Meta:
        model = StatutoryDeduction
        fields = "__all__"
