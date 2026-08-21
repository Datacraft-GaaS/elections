"""Initial migration: creates test-side tables for unmanaged models.

In production the Rust Flyway migrations own the schema (managed=False).
This migration exists only so Django's test runner can create the tables in SQLite.
It is excluded from prod deployments via DJANGO_TEST_SQLITE guard.
"""

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):
    initial = True
    dependencies = []

    operations = [
        migrations.CreateModel(
            name="ReportDefinition",
            fields=[
                ("definition_id", models.UUIDField(primary_key=True, serialize=False)),
                ("slug", models.TextField(unique=True)),
                ("title", models.TextField()),
                ("report_class", models.TextField()),
                ("audience", models.JSONField()),
                ("data_class", models.TextField()),
                ("sql_template_ref", models.TextField(blank=True, null=True)),
                ("renderer_pipeline", models.JSONField()),
                ("requires_hsm_sign", models.BooleanField(default=False)),
                ("requires_tlog_anchor", models.BooleanField(default=False)),
                ("default_schedule", models.TextField(blank=True, null=True)),
                ("timezone", models.TextField(default="Africa/Nairobi")),
                ("retention_years", models.SmallIntegerField(default=7)),
                ("template_version", models.TextField()),
                ("cosign_signature", models.BinaryField(blank=True, null=True)),
                ("created_at", models.DateTimeField()),
                ("updated_at", models.DateTimeField()),
                ("retired_at", models.DateTimeField(blank=True, null=True)),
                ("retirement_reason", models.TextField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_definitions"},
        ),
        migrations.CreateModel(
            name="ReportRun",
            fields=[
                ("run_id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "definition",
                    models.ForeignKey(
                        db_column="definition_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportdefinition",
                    ),
                ),
                ("slug", models.TextField()),
                ("window_from", models.DateTimeField()),
                ("window_to", models.DateTimeField()),
                ("status", models.TextField(default="queued")),
                ("requested_by", models.TextField()),
                ("requested_at", models.DateTimeField()),
                ("started_at", models.DateTimeField(blank=True, null=True)),
                ("completed_at", models.DateTimeField(blank=True, null=True)),
                ("rows_extracted", models.BigIntegerField(blank=True, null=True)),
                ("extraction_ms", models.IntegerField(blank=True, null=True)),
                ("render_ms", models.IntegerField(blank=True, null=True)),
                ("total_ms", models.IntegerField(blank=True, null=True)),
                ("failure_reason", models.TextField(blank=True, null=True)),
                ("correlation_id", models.UUIDField(blank=True, null=True)),
                ("trace_id", models.BinaryField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_runs"},
        ),
        migrations.CreateModel(
            name="ReportArtifact",
            fields=[
                ("artifact_id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "run",
                    models.ForeignKey(
                        db_column="run_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        related_name="artifacts",
                        to="eel_reports.reportrun",
                    ),
                ),
                ("format", models.TextField()),
                ("blob_ref", models.TextField()),
                ("sha256", models.BinaryField()),
                ("size_bytes", models.BigIntegerField()),
                ("page_count", models.IntegerField(blank=True, null=True)),
                ("row_count", models.BigIntegerField(blank=True, null=True)),
                ("hsm_signature", models.BinaryField(blank=True, null=True)),
                ("hsm_key_role", models.TextField(blank=True, null=True)),
                ("rendered_at", models.DateTimeField()),
            ],
            options={"db_table": "eel_report_artifacts"},
        ),
        migrations.CreateModel(
            name="ReportTlogAnchor",
            fields=[
                ("anchor_id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "run",
                    models.ForeignKey(
                        db_column="run_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportrun",
                    ),
                ),
                (
                    "artifact",
                    models.ForeignKey(
                        db_column="artifact_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportartifact",
                    ),
                ),
                ("tlog_leaf_index", models.BigIntegerField()),
                ("tlog_root_hash", models.BinaryField()),
                ("inclusion_proof", models.BinaryField()),
                ("sth_size", models.BigIntegerField()),
                ("witness_cosigs", models.JSONField(default=list)),
                ("anchored_at", models.DateTimeField()),
            ],
            options={"db_table": "eel_report_tlog_anchors"},
        ),
        migrations.CreateModel(
            name="ReportSubscriber",
            fields=[
                ("subscriber_id", models.UUIDField(primary_key=True, serialize=False)),
                ("name", models.TextField()),
                ("channel", models.TextField()),
                ("endpoint", models.TextField()),
                ("endpoint_hash", models.BinaryField()),
                ("audience", models.TextField()),
                ("auth_ref", models.TextField(blank=True, null=True)),
                ("pgp_pubkey_ref", models.TextField(blank=True, null=True)),
                ("preferred_format", models.TextField(default="pdf")),
                ("created_at", models.DateTimeField()),
                ("verified_at", models.DateTimeField(blank=True, null=True)),
                ("suspended_at", models.DateTimeField(blank=True, null=True)),
                ("suspension_reason", models.TextField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_subscribers"},
        ),
        migrations.CreateModel(
            name="ReportSubscription",
            fields=[
                (
                    "subscription_id",
                    models.UUIDField(primary_key=True, serialize=False),
                ),
                (
                    "subscriber",
                    models.ForeignKey(
                        db_column="subscriber_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportsubscriber",
                    ),
                ),
                (
                    "definition",
                    models.ForeignKey(
                        db_column="definition_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportdefinition",
                    ),
                ),
                ("format", models.TextField()),
                ("filter", models.JSONField(default=dict)),
                ("created_at", models.DateTimeField()),
                ("cancelled_at", models.DateTimeField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_subscriptions"},
        ),
        migrations.CreateModel(
            name="ReportDelivery",
            fields=[
                ("delivery_id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "run",
                    models.ForeignKey(
                        db_column="run_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportrun",
                    ),
                ),
                (
                    "subscription",
                    models.ForeignKey(
                        db_column="subscription_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportsubscription",
                    ),
                ),
                (
                    "artifact",
                    models.ForeignKey(
                        db_column="artifact_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportartifact",
                    ),
                ),
                ("status", models.TextField(default="pending")),
                ("attempt_count", models.SmallIntegerField(default=0)),
                ("max_attempts", models.SmallIntegerField(default=5)),
                ("next_attempt_at", models.DateTimeField(blank=True, null=True)),
                ("last_attempt_at", models.DateTimeField(blank=True, null=True)),
                ("delivered_at", models.DateTimeField(blank=True, null=True)),
                ("last_error", models.TextField(blank=True, null=True)),
                ("remote_receipt_id", models.TextField(blank=True, null=True)),
                ("remote_receipt_at", models.DateTimeField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_deliveries"},
        ),
        migrations.CreateModel(
            name="ReportAccessLog",
            fields=[
                ("access_id", models.UUIDField(primary_key=True, serialize=False)),
                (
                    "artifact",
                    models.ForeignKey(
                        db_column="artifact_id",
                        on_delete=django.db.models.deletion.PROTECT,
                        to="eel_reports.reportartifact",
                    ),
                ),
                ("subject_id", models.TextField(blank=True, null=True)),
                ("ip_prefix", models.TextField(blank=True, null=True)),
                ("user_agent_hash", models.BinaryField(blank=True, null=True)),
                ("accessed_at", models.DateTimeField()),
                ("byte_range", models.TextField(blank=True, null=True)),
            ],
            options={"db_table": "eel_report_access_log"},
        ),
    ]
