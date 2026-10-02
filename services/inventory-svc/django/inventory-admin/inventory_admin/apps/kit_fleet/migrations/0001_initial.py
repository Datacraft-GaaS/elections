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
