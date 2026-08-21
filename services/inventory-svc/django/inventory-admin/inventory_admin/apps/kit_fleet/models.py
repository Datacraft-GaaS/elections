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
