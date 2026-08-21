from django.db import models


class ReservePool(models.Model):
    pool_id = models.CharField(max_length=64, primary_key=True)
    scope = models.CharField(max_length=128)
    asset_type = models.CharField(max_length=64)
    count = models.IntegerField(default=0)
    alert_threshold = models.IntegerField()
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        verbose_name = "Reserve Pool"
        verbose_name_plural = "Reserve Pools"

    def __str__(self) -> str:
        return f"{self.pool_id} ({self.asset_type})"
