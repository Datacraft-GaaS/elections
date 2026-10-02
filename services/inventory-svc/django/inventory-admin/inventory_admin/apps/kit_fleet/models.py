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

from django.db import models


class Kit(models.Model):
    class Status(models.TextChoices):
        NEW = "new", "New"
        IN_TRANSIT = "in_transit", "In Transit"
        IN_WAREHOUSE = "in_warehouse", "In Warehouse"
        PROVISIONED = "provisioned", "Provisioned"
        DEPLOYED = "deployed", "Deployed"
        RETURNED = "returned", "Returned"
        FAULTY = "faulty", "Faulty"
        DECOMMISSIONED = "decommissioned", "Decommissioned"

    kit_id = models.CharField(max_length=64, primary_key=True)
    serial = models.CharField(max_length=128)
    kit_type = models.CharField(max_length=64)
    status = models.CharField(max_length=32, choices=Status.choices, default=Status.NEW)
    station_code = models.CharField(max_length=64, blank=True, null=True)
    last_seen_at = models.DateTimeField(auto_now=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        verbose_name = "Kit"
        verbose_name_plural = "Kits"
        ordering = ["-last_seen_at"]

    def __str__(self) -> str:
        return f"{self.kit_id} ({self.kit_type})"
