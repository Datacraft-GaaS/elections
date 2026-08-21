"""Tests for the gov_printer delivery module."""

from __future__ import annotations

import hashlib

import pytest
from lxml import etree

from eel_report_deliverers.gov_printer import (
    GAZETTE_NS,
    build_gazette_xml,
    envelope_sha256,
)
from eel_report_deliverers.models import DeliveryRequest


def make_req() -> DeliveryRequest:
    return DeliveryRequest(
        artifact_ref="http://minio.internal/reports/abc.pdf",
        artifact_id="artifact-uuid-001",
        subscription_id="sub-uuid-001",
        format="pdf",
        sha256="deadbeef" * 8,
    )


def test_build_gazette_xml_valid_structure() -> None:
    req = make_req()
    xml_bytes = build_gazette_xml(req)
    root = etree.fromstring(xml_bytes)

    assert root.tag == f"{{{GAZETTE_NS}}}GazetteSubmission"
    meta = root.find(f"{{{GAZETTE_NS}}}SubmissionMeta")
    assert meta is not None

    artefact = root.find(f"{{{GAZETTE_NS}}}Artefact")
    assert artefact is not None

    artifact_id_el = artefact.find(f"{{{GAZETTE_NS}}}ArtifactId")
    assert artifact_id_el is not None
    assert artifact_id_el.text == req.artifact_id


def test_build_gazette_xml_contains_sha256() -> None:
    req = make_req()
    xml_bytes = build_gazette_xml(req)
    root = etree.fromstring(xml_bytes)
    artefact = root.find(f"{{{GAZETTE_NS}}}Artefact")
    assert artefact is not None
    sha = artefact.find(f"{{{GAZETTE_NS}}}Sha256")
    assert sha is not None
    assert sha.text == req.sha256


def test_build_gazette_xml_contains_format() -> None:
    req = make_req()
    xml_bytes = build_gazette_xml(req)
    root = etree.fromstring(xml_bytes)
    artefact = root.find(f"{{{GAZETTE_NS}}}Artefact")
    assert artefact is not None
    fmt = artefact.find(f"{{{GAZETTE_NS}}}Format")
    assert fmt is not None
    assert fmt.text == "pdf"


def test_envelope_sha256_matches_computed() -> None:
    req = make_req()
    xml_bytes = build_gazette_xml(req)
    expected = hashlib.sha256(xml_bytes).hexdigest()
    assert envelope_sha256(xml_bytes) == expected


def test_envelope_sha256_is_64_hex_chars() -> None:
    req = make_req()
    xml_bytes = build_gazette_xml(req)
    sha = envelope_sha256(xml_bytes)
    assert len(sha) == 64
    assert all(c in "0123456789abcdef" for c in sha)


@pytest.mark.asyncio
async def test_deliver_gov_printer_mock(httpserver: pytest.FixtureRequest) -> None:
    """Mock the Gov Printer HTTP endpoint and verify deliver() returns a receipt."""
    import os

    from pytest_httpserver import HTTPServer  # type: ignore[import-untyped]

    server: HTTPServer = httpserver  # type: ignore[assignment]
    server.expect_request("/gazette/submit", method="POST").respond_with_json(
        {"submission_id": "sub-receipt-001"}
    )

    os.environ["EEL_GOV_PRINTER_URL"] = server.url_for("/gazette/submit")
    req = make_req()

    from eel_report_deliverers.gov_printer import deliver

    receipt = await deliver(req)
    assert receipt.submission_id == "sub-receipt-001"
    assert receipt.submitted_at is not None
