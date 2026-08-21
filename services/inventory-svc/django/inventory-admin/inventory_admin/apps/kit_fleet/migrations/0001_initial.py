from django.db import migrations, models


class Migration(migrations.Migration):
    initial = True
    dependencies = []

    operations = [
        migrations.CreateModel(
            name="Kit",
            fields=[
                (
                    "kit_id",
                    models.CharField(max_length=64, primary_key=True, serialize=False),
                ),
                ("serial", models.CharField(max_length=128)),
                ("kit_type", models.CharField(max_length=64)),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("new", "New"),
                            ("in_transit", "In Transit"),
                            ("in_warehouse", "In Warehouse"),
                            ("provisioned", "Provisioned"),
                            ("deployed", "Deployed"),
                            ("returned", "Returned"),
                            ("faulty", "Faulty"),
                            ("decommissioned", "Decommissioned"),
                        ],
                        default="new",
                        max_length=32,
                    ),
                ),
                (
                    "station_code",
                    models.CharField(blank=True, max_length=64, null=True),
                ),
                ("last_seen_at", models.DateTimeField(auto_now=True)),
                ("created_at", models.DateTimeField(auto_now_add=True)),
            ],
            options={
                "verbose_name": "Kit",
                "verbose_name_plural": "Kits",
                "ordering": ["-last_seen_at"],
            },
        ),
    ]
