"""Tests for the kbc_sftp delivery module."""

from __future__ import annotations

import hashlib
import io
import tarfile

import pytest

from eel_report_deliverers.kbc_sftp import _build_filename, _pgp_sign, build_bundle
from eel_report_deliverers.models import DeliveryRequest


def make_req() -> DeliveryRequest:
    return DeliveryRequest(
        artifact_ref="http://minio.internal/reports/abc.pdf",
        artifact_id="artifact-uuid-002",
        subscription_id="sub-uuid-002",
        format="pdf",
        sha256="cafebabe" * 8,
    )


def _fake_signer(data: bytes) -> bytes:
    return b"-----BEGIN PGP SIGNATURE-----\nFAKE-SIG\n-----END PGP SIGNATURE-----\n"


def test_build_bundle_roundtrip() -> None:
    req = make_req()
    artifact_bytes = b"fake pdf content for testing"
    bundle_bytes, sha256_hex = build_bundle(req, artifact_bytes, signer=_fake_signer)

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
    bundle_bytes, sha256_hex = build_bundle(req, artifact_bytes, signer=_fake_signer)
    expected = hashlib.sha256(bundle_bytes).hexdigest()
    assert sha256_hex == expected


def test_build_bundle_manifest_contains_artifact_id() -> None:
    import json

    req = make_req()
    artifact_bytes = b"content"
    bundle_bytes, _ = build_bundle(req, artifact_bytes, signer=_fake_signer)

    buf = io.BytesIO(bundle_bytes)
    with tarfile.open(fileobj=buf, mode="r:gz") as tar:
        f = tar.extractfile("manifest.json")
        assert f is not None
        manifest = json.loads(f.read())

    assert manifest["artifact_id"] == req.artifact_id
    assert manifest["format"] == req.format


def test_pgp_sign_requires_gnupg() -> None:
    # Without python-gnupg installed (or a configured key), signing must fail
    # loudly rather than emit a worthless placeholder signature.
    with pytest.raises(RuntimeError):
        _pgp_sign(b"data")


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


async def test_deliver_uploads_bundle_and_returns_receipt(monkeypatch) -> None:
    # build_bundle is patched because its default signer requires a real
    # GnuPG keyring; deliver() resolves it from module globals at call time.
    from eel_report_deliverers import kbc_sftp

    async def fake_fetch(ref: str) -> bytes:
        return b"artifact payload"

    def fake_build_bundle(req: DeliveryRequest, artifact_bytes: bytes) -> tuple[bytes, str]:
        return b"bundle-bytes", "feedface" * 4

    def fake_upload(
        host: str,
        port: int,
        username: str,
        key_path: str,
        remote_dir: str,
        filename: str,
        data: bytes,
    ) -> str:
        captured["host"] = host
        captured["filename"] = filename
        captured["data"] = data
        return f"{remote_dir}/{filename}"

    req = make_req()
    captured: dict = {}

    monkeypatch.setattr(kbc_sftp, "_fetch_artifact", fake_fetch)
    monkeypatch.setattr(kbc_sftp, "build_bundle", fake_build_bundle)
    monkeypatch.setattr(kbc_sftp, "_sftp_upload", fake_upload)

    receipt = await kbc_sftp.deliver(req)

    assert receipt.remote_path == f"/uploads/eel/{_build_filename(req)}"
    assert receipt.uploaded_at is not None
    assert captured["filename"] == _build_filename(req)
    assert captured["data"] == b"bundle-bytes"
    assert receipt.sha256 == "feedface" * 4
def test_pgp_sign_unavailable_flag_matches_runtime() -> None:
    """The availability flag must reflect the interpreter, not a hardcoded False."""
    import importlib.util

    from eel_report_deliverers import kbc_sftp

    expected = importlib.util.find_spec("gnupg") is not None
    assert kbc_sftp._gnupg_available[0] == expected
