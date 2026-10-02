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

import base64
from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from eel_secrets.client import OpenBaoClient
from eel_secrets.dynamic_creds import PgCredential, PgDynamicCred
from eel_secrets.transit import Transit


# -- Transit encrypt/decrypt round-trip (mocked client) --


async def test_transit_encrypt_decrypt_roundtrip(respx_mock):
    import httpx

    plaintext = b"hello secret world"
    b64_plaintext = base64.b64encode(plaintext).decode()
    fake_cipher = "vault:v1:encryptedstuff"

    respx_mock.post("http://bao.example.com/v1/transit/encrypt/mykey").mock(
        return_value=httpx.Response(200, json={"data": {"ciphertext": fake_cipher}})
    )
    respx_mock.post("http://bao.example.com/v1/transit/decrypt/mykey").mock(
        return_value=httpx.Response(200, json={"data": {"plaintext": b64_plaintext}})
    )

    async with httpx.AsyncClient() as http_client:
        client = OpenBaoClient(
            base_url="http://bao.example.com",
            token="root-token",
            client=http_client,
        )
        transit = Transit(client)
        ciphertext = await transit.encrypt("mykey", plaintext)
        assert ciphertext == fake_cipher

        recovered = await transit.decrypt("mykey", ciphertext)
        assert recovered == plaintext


# -- PgDynamicCred.connection_string format --


def test_pg_connection_string_format():
    cred = PgCredential(
        username="v-approle-myapp-abc",
        password="s3cr3t",
        lease_id="lease/db/123",
        lease_duration_s=3600,
    )
    client = OpenBaoClient(base_url="http://bao.example.com", token="t")
    pg = PgDynamicCred(client, db_role="myapp-role")
    cs = pg.connection_string(cred, host="db.example.com", port=5432, dbname="mydb")
    assert cs == "postgresql://v-approle-myapp-abc:s3cr3t@db.example.com:5432/mydb"


# -- PgDynamicCred.MAX_LEASE_SECONDS --


def test_max_lease_seconds():
    assert PgDynamicCred.MAX_LEASE_SECONDS == 14400  # 4 * 3600


# -- OpenBaoClient._ensure_authed calls auth_approle if no token --


async def test_ensure_authed_calls_approle_when_no_token(respx_mock):
    import httpx

    respx_mock.post("http://bao.example.com/v1/auth/approle/login").mock(
        return_value=httpx.Response(
            200,
            json={
                "auth": {
                    "client_token": "new-token-xyz",
                    "lease_duration": 3600,
                }
            },
        )
    )

    async with httpx.AsyncClient() as http_client:
        client = OpenBaoClient(
            base_url="http://bao.example.com",
            role_id="role-abc",
            secret_id="secret-xyz",
            client=http_client,
        )
        assert client._token is None
        await client._ensure_authed()
        assert client._token == "new-token-xyz"


async def test_ensure_authed_skips_approle_when_token_set():
    """_ensure_authed must not call auth_approle if a non-expiring token is already set."""
    client = OpenBaoClient(
        base_url="http://bao.example.com",
        token="existing-token",
    )
    # token_expires_at is 0.0 (infinite), so no renewal needed
    original_auth = client.auth_approle
    called = False

    async def spy(*args: Any, **kwargs: Any) -> str:
        nonlocal called
        called = True
        return await original_auth(*args, **kwargs)

    client.auth_approle = spy  # type: ignore[method-assign]
    await client._ensure_authed()
    assert not called
