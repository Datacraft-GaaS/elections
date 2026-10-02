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


class BallotLot(models.Model):
    class Status(models.TextChoices):
        PENDING = "pending", "Pending"
        DUAL_SIGNED = "dual_signed", "Dual Signed"
        TRANSMITTED = "transmitted", "Transmitted"
        PRINTED = "printed", "Printed"
        IN_TRANSIT = "in_transit", "In Transit"
        AT_WAREHOUSE = "at_warehouse", "At Warehouse"
        ALLOCATED = "allocated", "Allocated"
        AT_STATION = "at_station", "At Station"
        RECONCILED = "reconciled", "Reconciled"

    lot_id = models.CharField(max_length=64, primary_key=True)
    print_order_id = models.CharField(max_length=64)
    printer_facility = models.CharField(max_length=128)
    serial_range_start = models.BigIntegerField()
    serial_range_end = models.BigIntegerField()
    quantity_printed = models.BigIntegerField()
    variance_pct = models.FloatField(default=0.0)
    status = models.CharField(
        max_length=32, choices=Status.choices, default=Status.PENDING
    )
    received_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        verbose_name = "Ballot Lot"
        verbose_name_plural = "Ballot Lots"

    def __str__(self) -> str:
        return f"{self.lot_id} ({self.status})"
