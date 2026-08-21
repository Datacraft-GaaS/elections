"""DRF viewsets for M55 HR-Payroll (WP-5.6)."""

from rest_framework import viewsets
from rest_framework.response import Response

from .models import (
    JobApplication,
    PayrollRun,
    PayslipLine,
    StaffContract,
    StatutoryDeduction,
)
from .serializers import (
    JobApplicationSerializer,
    PayrollRunSerializer,
    PayslipLineSerializer,
    StaffContractSerializer,
    StatutoryDeductionSerializer,
)


class JobApplicationViewSet(viewsets.ModelViewSet):
    queryset = JobApplication.objects.all()
    serializer_class = JobApplicationSerializer


class StaffContractViewSet(viewsets.ModelViewSet):
    queryset = StaffContract.objects.all()
    serializer_class = StaffContractSerializer


class PayrollRunViewSet(viewsets.ModelViewSet):
    queryset = PayrollRun.objects.all()
    serializer_class = PayrollRunSerializer


class PayslipLineViewSet(viewsets.ModelViewSet):
    queryset = PayslipLine.objects.all()
    serializer_class = PayslipLineSerializer


class StatutoryDeductionViewSet(viewsets.ModelViewSet):
    queryset = StatutoryDeduction.objects.all()
    serializer_class = StatutoryDeductionSerializer
