"""Tests for the kbc_sftp delivery module."""

from __future__ import annotations

import hashlib
import io
import tarfile

import pytest

from eel_report_deliverers.kbc_sftp import _build_filename, _pgp_sign_stub, build_bundle
from eel_report_deliverers.models import DeliveryRequest


def make_req() -> DeliveryRequest:
    return DeliveryRequest(
        artifact_ref="http://minio.internal/reports/abc.pdf",
        artifact_id="artifact-uuid-002",
        subscription_id="sub-uuid-002",
        format="pdf",
        sha256="cafebabe" * 8,
    )


def test_build_bundle_roundtrip() -> None:
    req = make_req()
    artifact_bytes = b"fake pdf content for testing"
    bundle_bytes, sha256_hex = build_bundle(req, artifact_bytes)

    # Verify the bundle is a valid gzipped tar.
    buf = io.BytesIO(bundle_bytes)
    with tarfile.open(fileobj=buf, mode="r:gz") as tar:
        names = tar.getnames()

    assert f"{req.artifact_id}.{req.format}" in names
    assert "manifest.json" in names
    assert "manifest.sig" in names


def test_build_bundle_sha256_matches() -> None:
    req = make_req()
    artifact_bytes = b"report bytes"
    bundle_bytes, sha256_hex = build_bundle(req, artifact_bytes)
    expected = hashlib.sha256(bundle_bytes).hexdigest()
    assert sha256_hex == expected


def test_build_bundle_manifest_contains_artifact_id() -> None:
    import json

    req = make_req()
    artifact_bytes = b"content"
    bundle_bytes, _ = build_bundle(req, artifact_bytes)

    buf = io.BytesIO(bundle_bytes)
    with tarfile.open(fileobj=buf, mode="r:gz") as tar:
        f = tar.extractfile("manifest.json")
        assert f is not None
        manifest = json.loads(f.read())

    assert manifest["artifact_id"] == req.artifact_id
    assert manifest["format"] == req.format


def test_pgp_sign_stub_returns_placeholder() -> None:
    sig = _pgp_sign_stub(b"data")
    assert b"PGP SIGNATURE" in sig


def test_build_filename_convention() -> None:
    req = make_req()
    name = _build_filename(req)
    assert name.startswith("eel_")
    assert req.subscription_id in name
    assert req.artifact_id in name
    assert name.endswith(".tar.gz")


def test_sftp_path_builds_correctly() -> None:
    req = make_req()
    remote_dir = "/uploads/eel"
    filename = _build_filename(req)
    remote_path = f"{remote_dir}/{filename}"
    assert remote_path.startswith("/uploads/eel/eel_")
    assert remote_path.endswith(".tar.gz")
