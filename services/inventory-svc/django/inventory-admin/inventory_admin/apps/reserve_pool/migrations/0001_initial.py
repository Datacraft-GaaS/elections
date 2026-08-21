from django.db import migrations, models


class Migration(migrations.Migration):
    initial = True
    dependencies = []

    operations = [
        migrations.CreateModel(
            name="ReservePool",
            fields=[
                (
                    "pool_id",
                    models.CharField(max_length=64, primary_key=True, serialize=False),
                ),
                ("scope", models.CharField(max_length=128)),
                ("asset_type", models.CharField(max_length=64)),
                ("count", models.IntegerField(default=0)),
                ("alert_threshold", models.IntegerField()),
                ("updated_at", models.DateTimeField(auto_now=True)),
            ],
            options={
                "verbose_name": "Reserve Pool",
                "verbose_name_plural": "Reserve Pools",
            },
        ),
    ]
