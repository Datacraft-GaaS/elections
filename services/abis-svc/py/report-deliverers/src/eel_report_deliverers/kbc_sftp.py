"""KBC SFTP channel: bundles artefact + manifest + PGP stub into tar.gz and uploads via paramiko."""

from __future__ import annotations

import asyncio
import hashlib
import io
import json
import os
import tarfile
from datetime import datetime, timezone

import httpx
import paramiko

from .models import DeliveryReceipt, DeliveryRequest

# Use a mutable list-of-one so pyright does not treat it as a redefined constant.
_gnupg_available: list[bool] = [False]
try:
    import gnupg  # type: ignore[import-untyped]  # noqa: F401

    _gnupg_available[0] = True
except ImportError:
    pass


def _pgp_sign_stub(data: bytes) -> bytes:
    """Stub PGP signature: returns a clearly-marked placeholder."""
    if not _gnupg_available[0]:
        # Return a placeholder signature block so the tar.gz can always be built.
        return (
            b"-----BEGIN PGP SIGNATURE-----\n"
            b"(stub: gnupg not installed; install python-gnupg for real signatures)\n"
            b"-----END PGP SIGNATURE-----\n"
        )
    raise NotImplementedError(
        "full PGP sign not yet wired; install python-gnupg and configure key"
    )


def build_bundle(req: DeliveryRequest, artifact_bytes: bytes) -> tuple[bytes, str]:
    """Bundle artifact + manifest + PGP sig into a gzipped tar archive.

    Returns (bundle_bytes, bundle_sha256_hex).
    """
    manifest = {
        "artifact_id": req.artifact_id,
        "subscription_id": req.subscription_id,
        "format": req.format,
        "sha256": req.sha256,
        "bundled_at": datetime.now(timezone.utc).isoformat(),
        "system": "eel-abis",
    }
    manifest_bytes = json.dumps(manifest, indent=2).encode()
    sig_bytes = _pgp_sign_stub(artifact_bytes)

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:

        def _add(name: str, content: bytes) -> None:
            info = tarfile.TarInfo(name=name)
            info.size = len(content)
            tar.addfile(info, io.BytesIO(content))

        _add(f"{req.artifact_id}.{req.format}", artifact_bytes)
        _add("manifest.json", manifest_bytes)
        _add("manifest.sig", sig_bytes)

    bundle = buf.getvalue()
    sha256_hex = hashlib.sha256(bundle).hexdigest()
    return bundle, sha256_hex


async def _fetch_artifact(artifact_ref: str) -> bytes:
    async with httpx.AsyncClient(timeout=60.0) as client:
        resp = await client.get(artifact_ref)
        resp.raise_for_status()
        return resp.content


def _sftp_upload(
    host: str,
    port: int,
    username: str,
    key_path: str,
    remote_dir: str,
    filename: str,
    data: bytes,
) -> str:
    """Upload bytes via paramiko SFTP and return the remote path."""
    key = paramiko.RSAKey.from_private_key_file(key_path)
    with paramiko.SSHClient() as ssh:
        ssh.set_missing_host_key_policy(paramiko.RejectPolicy())
        ssh.connect(
            hostname=host, port=port, username=username, pkey=key, look_for_keys=False
        )
        with ssh.open_sftp() as sftp:
            remote_path = f"{remote_dir}/{filename}"
            with sftp.file(remote_path, "wb") as f:
                f.write(data)
    return remote_path


def _build_filename(req: DeliveryRequest) -> str:
    """KBC file naming convention: eel_{subscription_id}_{artifact_id}.tar.gz"""
    return f"eel_{req.subscription_id}_{req.artifact_id}.tar.gz"


async def deliver(req: DeliveryRequest) -> DeliveryReceipt:
    """Bundle and upload artifact to KBC SFTP."""
    host = os.environ.get("EEL_KBC_SFTP_HOST", "sftp.kbc.example.go.ke")
    port = int(os.environ.get("EEL_KBC_SFTP_PORT", "22"))
    username = os.environ.get("EEL_KBC_SFTP_USER", "eel")
    key_path = os.environ.get("EEL_KBC_SFTP_KEY_PATH", "")
    remote_dir = os.environ.get("EEL_KBC_SFTP_REMOTE_DIR", "/uploads/eel")

    artifact_bytes = await _fetch_artifact(req.artifact_ref)
    bundle, sha256_hex = build_bundle(req, artifact_bytes)
    filename = _build_filename(req)

    # Run blocking SFTP upload in thread executor.
    remote_path: str = await asyncio.get_event_loop().run_in_executor(
        None,
        lambda: _sftp_upload(
            host, port, username, key_path, remote_dir, filename, bundle
        ),
    )

    return DeliveryReceipt(
        remote_path=remote_path,
        sha256=sha256_hex,
        uploaded_at=datetime.now(timezone.utc),
    )
