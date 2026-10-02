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

"""Tests for beacon ingestion, signature verification, and seed mixing.

No network: all HTTP goes through httpx.MockTransport with canned records,
and signatures are produced against a locally generated RSA key.
"""

import base64

import httpx
import pytest
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa

from shangrla.beacon import (
    BeaconClient,
    build_signed_message,
    combine_seeds,
    commit_seed,
    parse_pulse_record,
    verify_commitment,
)


def _make_key() -> rsa.RSAPrivateKey:
    return rsa.generate_private_key(public_exponent=65537, key_size=2048)


def _canned_record() -> dict:
    return {
        "uri": "https://beacon.nist.gov/beacon/2.0/pulse/12345",
        "version": "Beacon 2.0",
        "cipherSuite": 1,
        "frequency": 60000,
        "periodic": True,
        "pulseIndex": 12345,
        "localRandomValue": "aa" * 32,
        "outputValue": "cc" * 32,
        "timeStamp": "2026-08-25T00:00:00.000Z",
        "previousOutputValue": "bb" * 32,
        "signatureValue": "",  # filled by signer in tests that verify
    }


def _sign(private_key: rsa.RSAPrivateKey, record: dict) -> str:
    signature = private_key.sign(
        build_signed_message(record),
        padding.PKCS1v15(),
        hashes.SHA512(),
    )
    return base64.b64encode(signature).decode()


def _make_client(record: dict) -> BeaconClient:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path.endswith(("/last", f"/{record['pulseIndex']}"))
        return httpx.Response(200, json=record)

    return BeaconClient(
        transport=httpx.MockTransport(handler),
        public_key_pem_path=None,
    )


class TestFetchPulse:
    def test_fetch_parses_canned_v2_record(self):
        record = _canned_record()
        client = _make_client(record)
        try:
            pulse = client.fetch_pulse()
            assert pulse.pulse_index == 12345
            assert pulse.output_value == "cc" * 32
            assert len(pulse.output_value) == 64
            assert pulse.fetched_at
            # raw record retained for verification
            assert client._last_record == record
        finally:
            client.close()

    def test_fetch_by_index(self):
        record = _canned_record()
        client = _make_client(record)
        try:
            pulse = client.fetch_pulse(12345)
            assert pulse.pulse_index == 12345
        finally:
            client.close()

    def test_parse_rejects_bad_output_value(self):
        record = _canned_record()
        record["outputValue"] = "zz" * 32
        with pytest.raises(ValueError):
            parse_pulse_record(record, fetched_at="now")


class TestSignatureVerification:
    def test_verify_round_trip_with_local_key(self, tmp_path):
        private_key = _make_key()
        key_path = tmp_path / "beacon_pub.pem"
        key_path.write_bytes(
            private_key.public_key().public_bytes(
                serialization.Encoding.PEM,
                serialization.PublicFormat.SubjectPublicKeyInfo,
            )
        )
        record = _canned_record()
        record["signature"] = _sign(private_key, record)

        client = _make_client(record)
        try:
            pulse = client.fetch_pulse()
            client.load_public_key(str(key_path))
            assert client.verify_signature(pulse, record) is True
        finally:
            client.close()

    def test_tampered_output_value_fails(self, tmp_path):
        private_key = _make_key()
        key_path = tmp_path / "beacon_pub.pem"
        key_path.write_bytes(
            private_key.public_key().public_bytes(
                serialization.Encoding.PEM,
                serialization.PublicFormat.SubjectPublicKeyInfo,
            )
        )
        record = _canned_record()
        record["signature"] = _sign(private_key, record)
        tampered = dict(record, localRandomValue="cc" * 32)

        client = _make_client(tampered)
        try:
            pulse = client.fetch_pulse()
            client.load_public_key(str(key_path))
            assert client.verify_signature(pulse, tampered) is False
        finally:
            client.close()

    def test_wrong_key_fails(self, tmp_path):
        signer = _make_key()
        other = _make_key()
        key_path = tmp_path / "other_pub.pem"
        key_path.write_bytes(
            other.public_key().public_bytes(
                serialization.Encoding.PEM,
                serialization.PublicFormat.SubjectPublicKeyInfo,
            )
        )
        record = _canned_record()
        record["signature"] = _sign(signer, record)
        client = _make_client(record)
        try:
            pulse = client.fetch_pulse()
            client.load_public_key(str(key_path))
            assert client.verify_signature(pulse, record) is False
        finally:
            client.close()

    def test_no_key_raises_never_silent_pass(self):
        record = _canned_record()
        record["signature"] = _sign(_make_key(), record)
        client = _make_client(record)
        try:
            pulse = client.fetch_pulse()
            with pytest.raises(ValueError, match="no public key"):
                client.verify_signature(pulse, record)
        finally:
            client.close()


class TestCombineSeeds:
    PARTIES = [("alpha", "11" * 32), ("beta", "22" * 32), ("gamma", "33" * 32)]
    BEACON_HEX = "de" * 32

    def test_deterministic(self):
        s1 = combine_seeds(self.BEACON_HEX, self.PARTIES)
        s2 = combine_seeds(self.BEACON_HEX, self.PARTIES)
        assert s1 == s2
        assert len(s1) == 64

    def test_order_independent(self):
        forward = combine_seeds(self.BEACON_HEX, self.PARTIES)
        backward = combine_seeds(self.BEACON_HEX, list(reversed(self.PARTIES)))
        shuffled = combine_seeds(self.BEACON_HEX, [self.PARTIES[2], self.PARTIES[0], self.PARTIES[1]])
        assert forward == backward == shuffled

    def test_changes_if_any_seed_changes(self):
        baseline = combine_seeds(self.BEACON_HEX, self.PARTIES)
        mutated = [(pid, seed if pid != "beta" else "ff" * 32) for pid, seed in self.PARTIES]
        assert combine_seeds(self.BEACON_HEX, mutated) != baseline
        assert combine_seeds("ab" * 32, self.PARTIES) != baseline

    def test_duplicate_party_id_rejected(self):
        dupes = [("alpha", "11" * 32), ("alpha", "22" * 32)]
        with pytest.raises(ValueError, match="duplicate party_id"):
            combine_seeds(self.BEACON_HEX, dupes)

    def test_commitment_checked_on_combine(self):
        commitments = [(pid, commit_seed(seed)) for pid, seed in self.PARTIES]
        ok = combine_seeds(self.BEACON_HEX, self.PARTIES, commitments)
        assert len(ok) == 64
        bad_reveals = [("alpha", "99" * 32)] + self.PARTIES[1:]
        with pytest.raises(ValueError, match="does not match its commitment"):
            combine_seeds(self.BEACON_HEX, bad_reveals, commitments)


class TestCommitments:
    def test_round_trip(self):
        seed = "42" * 32
        commitment = commit_seed(seed)
        assert verify_commitment(seed, commitment)
        assert not verify_commitment("43" * 32, commitment)

    def test_wrong_reveal_fails_verification(self):
        commitment = commit_seed("abcd")
        assert verify_commitment("abcd", commitment)
        assert not verify_commitment("abce", commitment)
