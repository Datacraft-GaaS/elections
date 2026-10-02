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

"""Verified randomness-beacon ingestion and multi-party seed mixing.

This module provides:

* ``BeaconClient`` — fetches NIST Randomness Beacon v2 pulses and verifies
  their RSA signatures against an injected public key.
* ``combine_seeds`` / ``commit_seed`` / ``verify_commitment`` — multi-party
  seed mixing with hash commitments, so no single party (or the audit
  administrator) can cherry-pick the sample seed.

Signature verification scope
----------------------------
The NIST Beacon v2 record schema carries many fields (uri, version,
cipherSuite, frequency, periodic, pulseIndex, timeStamp, localRandomValue,
prior-chain values, signatureValue). The official signed message is a
specific serialisation defined by NIST IR 8293 / the beacon v2 profile.
Rather than silently skipping verification when the full canonicalisation
is unavailable, this module implements verification over the following
explicitly-documented message construction (``build_signed_message``):

    message = b"NIST Beacon v2.0 Pulse "
            + profile_name.encode("utf-8")
            + pulse_index.to_bytes(8, "big")
            + time_stamp.encode("utf-8")          # ISO-8601 string as returned
            + bytes.fromhex(local_random_value)
            + bytes.fromhex(previous_output_value)  # chain binding; empty
                                                    # bytes when absent

The signature scheme verified is RSASSA-PKCS1-v1_5 with SHA-512
(``cryptography.hazmat.primitives.hashes.SHA512``), which matches the
default NIST v2 RSA/SHA-512 suite.

**What exactly is covered:** any pulse whose record supplies
``profileName``, ``pulseIndex``, ``timeStamp``, ``localRandomValue`` and
(optionally) the previous pulse's ``outputValue`` can be verified, provided
the signer used the identical concatenation above. **What is not covered:**
pulses whose real-world signing serialisation differs from this layout
(e.g. additional external/metadata fields mandated by a specific profile)
will fail verification rather than falsely pass — verification is strict,
never skipped. Callers must supply the NIST beacon public key PEM; without
a key ``verify_signature`` raises rather than returning True.
"""

from __future__ import annotations

import base64
import binascii
import datetime
import hashlib
import hmac
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any

import httpx
from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa

DEFAULT_NIST_BEACON_BASE_URL = "https://beacon.nist.gov/beacon/2.0/pulse"
_TYPE_STRING = b"NIST Beacon v2.0 Pulse "
_COMMITMENT_PREFIX = "eel-seed-commit:"
_DEFAULT_PROFILE_NAME = "Default"


@dataclass
class BeaconPulse:
    """A single verified randomness-beacon pulse."""

    pulse_index: int
    output_value: str  # 64-hex-char random value
    signature_b64: str
    fetched_at: str


def build_signed_message(
    record: Mapping[str, Any],
    profile_name: str = _DEFAULT_PROFILE_NAME,
) -> bytes:
    """Build the canonical byte string whose signature we verify.

    See the module docstring for the exact, exhaustive definition of this
    serialisation. This function is the single source of truth for the
    message format; signers and verifiers MUST both call it.
    """
    time_stamp = str(record.get("timeStamp", ""))
    local_random = str(record.get("localRandomValue", "") or "")
    previous = record.get("previous")
    if isinstance(previous, Mapping):
        previous_output = str(previous.get("outputValue", "") or "")
    else:
        previous_output = str(record.get("previousOutputValue", "") or "")
    return (
        _TYPE_STRING
        + str(profile_name).encode("utf-8")
        + int(record["pulseIndex"]).to_bytes(8, "big")
        + time_stamp.encode("utf-8")
        + _decode_hex(local_random)
        + _decode_hex(previous_output)
    )


def _decode_hex(hex_str: str) -> bytes:
    cleaned = hex_str.strip()
    if not cleaned:
        return b""
    try:
        return bytes.fromhex(cleaned)
    except ValueError as exc:
        raise ValueError(f"invalid hex field in beacon record: {exc}") from exc


def parse_pulse_record(record: Mapping[str, Any], fetched_at: str) -> BeaconPulse:
    """Parse a NIST Beacon v2-shaped JSON pulse record."""
    output_value = str(record["outputValue"])
    if len(output_value) != 64:
        raise ValueError("outputValue must be 64 hex characters")
    int(output_value, 16)  # validate hex
    return BeaconPulse(
        pulse_index=int(record["pulseIndex"]),
        output_value=output_value,
        signature_b64=str(record.get("signature", "")),
        fetched_at=fetched_at,
    )


class BeaconClient:
    """HTTP client for NIST Randomness Beacon v2 pulses.

    Args:
        base_url: Beacon endpoint root; pulses are read from
            ``{base_url}/last`` or ``{base_url}/{pulse_index}``.
        transport: Optional injected ``httpx.BaseTransport`` (e.g.
            ``httpx.MockTransport`` for tests).
        public_key_pem_path: Path to the beacon signer's RSA public key in
            PEM form. Required for ``verify_signature``.
    """

    def __init__(
        self,
        base_url: str = DEFAULT_NIST_BEACON_BASE_URL,
        transport: httpx.BaseTransport | None = None,
        public_key_pem_path: str | None = None,
    ):
        self.base_url = base_url.rstrip("/")
        self._client = httpx.Client(base_url=self.base_url, transport=transport)
        self._public_key: rsa.RSAPublicKey | None = None
        self._public_key_pem_path = public_key_pem_path
        self._last_record: Mapping[str, Any] | None = None

    def close(self) -> None:
        self._client.close()

    def __enter__(self) -> BeaconClient:
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    def fetch_pulse(self, pulse_index: int | None = None) -> BeaconPulse:
        """Fetch a pulse: the latest when ``pulse_index`` is None, else that index."""
        suffix = "last" if pulse_index is None else str(pulse_index)
        resp = self._client.get(f"/{suffix}")
        resp.raise_for_status()
        record = resp.json()
        self._last_record = record
        return parse_pulse_record(record, fetched_at=_utc_now_iso())

    def load_public_key(self, pem_path: str | None = None) -> rsa.RSAPublicKey:
        path = pem_path or self._public_key_pem_path
        if not path:
            raise ValueError(
                "no beacon public key configured: pass public_key_pem_path"
            )
        with open(path, "rb") as fh:
            key = serialization.load_pem_public_key(fh.read())
        if not isinstance(key, rsa.RSAPublicKey):
            raise ValueError("beacon public key must be an RSA key")
        self._public_key = key
        return key

    def verify_signature(
        self,
        pulse: BeaconPulse,
        record: Mapping[str, Any] | None = None,
        pem_path: str | None = None,
    ) -> bool:
        """Verify the pulse's RSA SHA-512 signature.

        Verifies RSASSA-PKCS1-v1_5/SHA-512 over exactly the message built by
        :func:`build_signed_message` (see module docstring for the precise
        byte-level format). Uses ``record`` when supplied, otherwise the raw
        record captured by the most recent ``fetch_pulse`` call. Raises
        ``ValueError`` if no public key or raw record is available — it never
        returns True without performing a real cryptographic check.
        """
        key = self._public_key
        if pem_path or (key is None and (pem_path or self._public_key_pem_path)):
            key = self.load_public_key(pem_path)
        if key is None:
            raise ValueError("cannot verify signature: no public key loaded")
        src = record if record is not None else self._last_record
        if src is None:
            raise ValueError(
                "cannot verify signature: no pulse record available "
                "(fetch a pulse or pass the raw record)"
            )
        message = build_signed_message(src)
        try:
            signature = base64.b64decode(str(pulse.signature_b64), validate=True)
        except (binascii.Error, ValueError) as exc:
            raise ValueError(f"signature is not valid base64: {exc}") from exc
        try:
            key.verify(
                signature,
                message,
                padding.PKCS1v15(),
                hashes.SHA512(),
            )
        except InvalidSignature:
            return False
        return True


def commit_seed(seed_hex: str) -> str:
    """Commit to a seed: sha256 over ``eel-seed-commit:<seed_hex>`` ASCII."""
    return hashlib.sha256((_COMMITMENT_PREFIX + seed_hex.strip()).encode()).hexdigest()


def verify_commitment(seed_hex: str, commitment: str) -> bool:
    """Check a revealed seed against its earlier commitment."""
    return hmac.compare_digest(commit_seed(seed_hex), commitment)


def _validate_commitments(party_commitments: list[tuple[str, str]]) -> None:
    seen: set[str] = set()
    for party_id, revealed_seed, commitment in party_commitments:
        if party_id in seen:
            raise ValueError(f"duplicate party_id: {party_id}")
        seen.add(party_id)
        if not verify_commitment(revealed_seed, commitment):
            raise ValueError(
                f"revealed seed for party {party_id!r} does not match its commitment"
            )


def combine_seeds(
    beacon_hex: str,
    party_reveals: list[tuple[str, str]],
    party_commitments: list[tuple[str, str]] | None = None,
) -> str:
    """Mix the beacon value with parties' revealed seeds into one final seed.

    ``party_reveals`` is a list of ``(party_id, revealed_seed_hex)``.
    When ``party_commitments`` (``(party_id, commitment_hex)``) is supplied,
    every reveal is checked against its commitment first and mixing fails
    closed on mismatch.

    Final seed = sha256(beacon_bytes || "party_id=seed" entries sorted
    lexicographically by party_id), so the result is independent of
    submission order and changes if any input changes.
    """
    beacon_bytes = _decode_hex(beacon_hex)
    reveals = [(pid, seed.strip()) for pid, seed in party_reveals]
    if len({pid for pid, _ in reveals}) != len(reveals):
        raise ValueError("duplicate party_id in reveals")
    if party_commitments is not None:
        _validate_commitments(
            [
                (pid, seed, cmt)
                for (pid, seed), (_, cmt) in zip(
                    sorted(reveals), sorted(party_commitments), strict=True
                )
            ]
        )
    h = hashlib.sha256()
    h.update(beacon_bytes)
    for party_id, seed in sorted(reveals):
        h.update(f"{party_id}={seed}".encode())
    return h.hexdigest()


def _utc_now_iso() -> str:
    return datetime.datetime.now(datetime.UTC).isoformat()


__all__ = [
    "DEFAULT_NIST_BEACON_BASE_URL",
    "BeaconClient",
    "BeaconPulse",
    "build_signed_message",
    "combine_seeds",
    "commit_seed",
    "parse_pulse_record",
    "verify_commitment",
]
