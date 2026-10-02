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
