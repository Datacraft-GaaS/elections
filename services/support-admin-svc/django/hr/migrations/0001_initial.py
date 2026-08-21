"""Initial migration for M55 HR-Payroll (WP-5.6)."""

import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):
    initial = True
    dependencies: list[tuple[str, str]] = []

    operations = [
        migrations.CreateModel(
            name="JobApplication",
            fields=[
                ("id", models.UUIDField(primary_key=True, serialize=False)),
                ("role_id", models.UUIDField()),
                ("applicant_name", models.CharField(max_length=255)),
                ("national_id_hash", models.CharField(max_length=64)),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("applied", "Applied"),
                            ("shortlisted", "Shortlisted"),
                            ("selected", "Selected"),
                            ("rejected", "Rejected"),
                        ],
                        default="applied",
                        max_length=20,
                    ),
                ),
                ("applied_at", models.DateTimeField()),
                ("training_gate_passed", models.BooleanField(default=False)),
            ],
            options={"db_table": "hr_job_application", "ordering": ["-applied_at"]},
        ),
        migrations.CreateModel(
            name="StaffContract",
            fields=[
                ("id", models.UUIDField(primary_key=True, serialize=False)),
                ("staff_id", models.UUIDField()),
                ("role", models.CharField(max_length=255)),
                ("station_id", models.UUIDField()),
                ("start_date", models.DateTimeField()),
                ("end_date", models.DateTimeField()),
                ("gross_salary_kes", models.BigIntegerField()),
                ("bank_account_hash", models.CharField(max_length=64)),
                ("deployment_ref", models.UUIDField(blank=True, null=True)),
            ],
            options={"db_table": "hr_staff_contract"},
        ),
        migrations.CreateModel(
            name="PayrollRun",
            fields=[
                ("id", models.UUIDField(primary_key=True, serialize=False)),
                ("election_id", models.UUIDField()),
                ("period", models.CharField(max_length=20)),
                (
                    "status",
                    models.CharField(
                        choices=[
                            ("draft", "Draft"),
                            ("approved", "Approved"),
                            ("paid", "Paid"),
                            ("failed", "Failed"),
                        ],
                        default="draft",
                        max_length=20,
                    ),
                ),
                ("total_staff", models.PositiveIntegerField(default=0)),
                ("total_gross_kes", models.BigIntegerField(default=0)),
                ("initiated_by", models.UUIDField()),
                ("approved_by", models.UUIDField(blank=True, null=True)),
                ("started_at", models.DateTimeField()),
                ("completed_at", models.DateTimeField(blank=True, null=True)),
            ],
            options={"db_table": "hr_payroll_run"},
        ),
        migrations.CreateModel(
            name="PayslipLine",
            fields=[
                ("id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "payroll_run",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="payslip_lines",
                        to="hr_app.payrollrun",
                    ),
                ),
                ("staff_id", models.UUIDField()),
                ("gross_kes", models.BigIntegerField()),
                ("paye_kes", models.BigIntegerField()),
                ("nssf_kes", models.BigIntegerField()),
                ("nhif_kes", models.BigIntegerField()),
                ("net_kes", models.BigIntegerField()),
            ],
            options={"db_table": "hr_payslip_line"},
        ),
        migrations.CreateModel(
            name="StatutoryDeduction",
            fields=[
                ("id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "payroll_run",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="statutory_deductions",
                        to="hr_app.payrollrun",
                    ),
                ),
                ("staff_id", models.UUIDField()),
                (
                    "deduction_type",
                    models.CharField(
                        choices=[
                            ("paye", "PAYE"),
                            ("nssf", "NSSF"),
                            ("nhif", "NHIF"),
                            ("shif", "SHIF"),
                        ],
                        max_length=10,
                    ),
                ),
                ("amount_kes", models.BigIntegerField()),
                ("reference", models.CharField(max_length=100)),
            ],
            options={"db_table": "hr_statutory_deduction"},
        ),
    ]
