import django.db.models.deletion
import django.utils.timezone
from django.db import migrations, models

import finance_app.models


class Migration(migrations.Migration):
    initial = True

    dependencies: list[tuple[str, str]] = []

    operations = [
        migrations.CreateModel(
            name="Budget",
            fields=[
                (
                    "id",
                    models.CharField(
                        default=finance_app.models.uuid7str,
                        max_length=36,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("election_id", models.CharField(max_length=36)),
                ("financial_year", models.CharField(max_length=20)),
                ("total_kes", models.BigIntegerField(default=0)),
                ("approved_by", models.CharField(blank=True, max_length=36, null=True)),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("draft", "Draft"),
                            ("approved", "Approved"),
                            ("active", "Active"),
                            ("closed", "Closed"),
                        ],
                        default="draft",
                        max_length=20,
                    ),
                ),
                ("created_at", models.DateTimeField(auto_now_add=True)),
                ("updated_at", models.DateTimeField(auto_now=True)),
            ],
            options={"ordering": ["-created_at"]},
        ),
        migrations.CreateModel(
            name="BudgetLine",
            fields=[
                (
                    "id",
                    models.CharField(
                        default=finance_app.models.uuid7str,
                        max_length=36,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                (
                    "budget",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="lines",
                        to="finance_app.budget",
                    ),
                ),
                ("programme_code", models.CharField(max_length=50)),
                ("description", models.TextField()),
                ("allocated_kes", models.BigIntegerField(default=0)),
                ("committed_kes", models.BigIntegerField(default=0)),
                ("spent_kes", models.BigIntegerField(default=0)),
                ("created_at", models.DateTimeField(auto_now_add=True)),
                ("updated_at", models.DateTimeField(auto_now=True)),
            ],
            options={"ordering": ["programme_code"]},
        ),
        migrations.CreateModel(
            name="ProcurementRecord",
            fields=[
                (
                    "id",
                    models.CharField(
                        default=finance_app.models.uuid7str,
                        max_length=36,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                (
                    "budget_line",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="procurements",
                        to="finance_app.budgetline",
                    ),
                ),
                ("vendor_id", models.CharField(max_length=36)),
                ("ppra_ref", models.CharField(max_length=100, unique=True)),
                ("amount_kes", models.BigIntegerField(default=0)),
                (
                    "procurement_method",
                    models.CharField(
                        choices=[
                            ("open_tender", "Open Tender"),
                            ("restricted_tender", "Restricted Tender"),
                            ("direct_procurement", "Direct Procurement"),
                            ("request_for_quotation", "Request for Quotation"),
                            ("low_value", "Low Value Procurement"),
                        ],
                        max_length=30,
                    ),
                ),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("pending", "Pending"),
                            ("awarded", "Awarded"),
                            ("cancelled", "Cancelled"),
                        ],
                        default="pending",
                        max_length=20,
                    ),
                ),
                ("created_at", models.DateTimeField(auto_now_add=True)),
                ("updated_at", models.DateTimeField(auto_now=True)),
            ],
            options={"ordering": ["-created_at"]},
        ),
        migrations.CreateModel(
            name="PaymentVoucher",
            fields=[
                (
                    "id",
                    models.CharField(
                        default=finance_app.models.uuid7str,
                        max_length=36,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                (
                    "procurement",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="vouchers",
                        to="finance_app.procurementrecord",
                    ),
                ),
                ("amount_kes", models.BigIntegerField(default=0)),
                ("initiated_by", models.CharField(max_length=36)),
                ("approved_by", models.CharField(blank=True, max_length=36, null=True)),
                (
                    "counter_approved_by",
                    models.CharField(blank=True, max_length=36, null=True),
                ),
                ("ifmis_ref", models.CharField(blank=True, max_length=100, null=True)),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("pending", "Pending"),
                            ("approved", "Approved"),
                            ("counter_approved", "Counter Approved"),
                            ("paid", "Paid"),
                            ("rejected", "Rejected"),
                        ],
                        default="pending",
                        max_length=20,
                    ),
                ),
                ("created_at", models.DateTimeField(auto_now_add=True)),
                ("updated_at", models.DateTimeField(auto_now=True)),
            ],
            options={"ordering": ["-created_at"]},
        ),
        migrations.CreateModel(
            name="CoBReport",
            fields=[
                (
                    "id",
                    models.CharField(
                        default=finance_app.models.uuid7str,
                        max_length=36,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                (
                    "budget",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="cob_reports",
                        to="finance_app.budget",
                    ),
                ),
                ("reporting_period", models.CharField(max_length=20)),
                ("total_committed", models.BigIntegerField(default=0)),
                ("total_spent", models.BigIntegerField(default=0)),
                ("variance", models.BigIntegerField(default=0)),
                ("submitted_at", models.DateTimeField(blank=True, null=True)),
                ("created_at", models.DateTimeField(auto_now_add=True)),
            ],
            options={
                "ordering": ["-created_at"],
                "unique_together": {("budget", "reporting_period")},
            },
        ),
    ]
