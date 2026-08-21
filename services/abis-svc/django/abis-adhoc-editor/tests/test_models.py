"""Tests for Django models (managed=False, so we test structure, not DB ops)."""

import uuid

import pytest

from adhoc_editor.apps.authoring.models import (
    AdhocPreview,
    AdhocReport,
    AdhocReportVersion,
    AdhocShare,
    AdhocSnippet,
)


def test_adhoc_report_str():
    r = AdhocReport(
        adhoc_id=uuid.uuid4(),
        slug="test-slug",
        title="Test Report",
        data_class="sensitive",
        visibility="private",
        author_id=uuid.uuid4(),
    )
    assert "Test Report" in str(r)
    assert "test-slug" in str(r)


def test_adhoc_report_meta():
    assert AdhocReport._meta.db_table == "eel_adhoc_reports"
    assert not AdhocReport._meta.managed


def test_adhoc_report_version_meta():
    assert AdhocReportVersion._meta.db_table == "eel_adhoc_report_versions"
    assert not AdhocReportVersion._meta.managed


def test_adhoc_preview_meta():
    assert AdhocPreview._meta.db_table == "eel_adhoc_previews"
    assert not AdhocPreview._meta.managed


def test_adhoc_share_meta():
    assert AdhocShare._meta.db_table == "eel_adhoc_shares"
    assert not AdhocShare._meta.managed


def test_adhoc_snippet_meta():
    assert AdhocSnippet._meta.db_table == "eel_adhoc_snippets"
    assert not AdhocSnippet._meta.managed


def test_data_class_choices():
    choices = dict(AdhocReport.DataClass.choices)
    assert "restricted" in choices
    assert "sensitive" in choices
    assert "public" in choices


def test_visibility_choices():
    choices = dict(AdhocReport.Visibility.choices)
    assert "private" in choices
    assert "shared" in choices
    assert "team" in choices
    assert "org" in choices


def test_snippet_kind_choices():
    choices = dict(AdhocSnippet.Kind.choices)
    assert "cte" in choices
    assert "where" in choices
    assert "select" in choices
    assert "markdown_partial" in choices


def test_share_permission_choices():
    choices = dict(AdhocShare.Permission.choices)
    assert "view" in choices
    assert "run" in choices
    assert "edit" in choices
    assert "admin" in choices


def test_report_version_meta():
    # Check ForeignKey points to right model
    fk = AdhocReportVersion._meta.get_field("adhoc")
    assert fk.related_model is AdhocReport


def test_preview_format_choices():
    choices = dict(AdhocPreview.PreviewFormat.choices)
    assert "json" in choices
    assert "markdown" in choices
    assert "docx" in choices
    assert "csv" in choices
