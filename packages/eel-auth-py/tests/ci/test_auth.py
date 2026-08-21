import asyncio
import logging
from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from eel_auth.audit import LogAuditSink
from eel_auth.jwks import AuthError, JwksCache, verify_token
from eel_auth.subject import Subject, SubjectExtractor


# -- SubjectExtractor --


def test_subject_extractor_basic():
    extractor = SubjectExtractor(realm="myrealm")
    claims: dict[str, Any] = {
        "sub": "user-123",
        "preferred_username": "alice",
        "email": "alice@example.com",
        "realm_access": {"roles": ["admin", "viewer"]},
    }
    subject = extractor.extract(claims)
    assert subject.sub == "user-123"
    assert subject.username == "alice"
    assert subject.email == "alice@example.com"
    assert "admin" in subject.roles
    assert subject.realm == "myrealm"
    assert subject.raw_claims == claims


def test_subject_extractor_missing_roles():
    extractor = SubjectExtractor(realm="test")
    claims: dict[str, Any] = {"sub": "u1"}
    subject = extractor.extract(claims)
    assert subject.roles == []
    assert subject.username is None
    assert subject.email is None


def test_subject_extractor_custom_roles_claim():
    extractor = SubjectExtractor(
        realm="test", roles_claim="resource_access.myapp.roles"
    )
    claims: dict[str, Any] = {
        "sub": "u2",
        "resource_access": {"myapp": {"roles": ["editor"]}},
    }
    subject = extractor.extract(claims)
    assert subject.roles == ["editor"]


# -- AuthError / verify_token --


async def test_verify_token_raises_auth_error_on_bad_token():
    cache = JwksCache(jwks_url="http://example.com/jwks")
    with pytest.raises(AuthError):
        await verify_token("not.a.valid.jwt", cache)


async def test_verify_token_raises_auth_error_on_missing_kid():
    import base64, json

    header = (
        base64.urlsafe_b64encode(json.dumps({"alg": "RS256"}).encode())
        .rstrip(b"=")
        .decode()
    )
    payload = (
        base64.urlsafe_b64encode(json.dumps({"sub": "x"}).encode())
        .rstrip(b"=")
        .decode()
    )
    token = f"{header}.{payload}.fakesig"
    cache = JwksCache(jwks_url="http://example.com/jwks")
    with pytest.raises(AuthError):
        await verify_token(token, cache)


# -- LogAuditSink --


async def test_log_audit_sink_logs(caplog):
    sink = LogAuditSink()
    subject = Subject(
        sub="u1",
        username="alice",
        email=None,
        roles=[],
        realm="test",
    )
    with caplog.at_level(logging.INFO, logger="eel_auth.audit"):
        await sink.record(subject, "read", "doc:123", "allow", {"ip": "1.2.3.4"})
    assert any("u1" in r.message for r in caplog.records)
    assert any("allow" in r.message for r in caplog.records)


# -- JwksCache kid-miss triggers fetch --


async def test_jwks_cache_fetches_on_kid_miss(respx_mock):
    import respx, httpx

    kid = "key-1"
    jwk_data = {"kid": kid, "kty": "RSA", "use": "sig", "n": "abc", "e": "AQAB"}
    respx_mock.get("http://auth.example.com/jwks").mock(
        return_value=httpx.Response(200, json={"keys": [jwk_data]})
    )

    async with httpx.AsyncClient() as client:
        cache = JwksCache(jwks_url="http://auth.example.com/jwks", client=client)
        key = await cache.get_key(kid)
    assert key["kid"] == kid


async def test_jwks_cache_raises_on_unknown_kid(respx_mock):
    import httpx

    respx_mock.get("http://auth.example.com/jwks").mock(
        return_value=httpx.Response(200, json={"keys": []})
    )
    async with httpx.AsyncClient() as client:
        cache = JwksCache(jwks_url="http://auth.example.com/jwks", client=client)
        with pytest.raises(AuthError, match="Unknown kid"):
            await cache.get_key("missing-kid")


# -- SpiceDBClient.check --


async def test_spicedb_check_allowed(respx_mock):
    import httpx
    from eel_auth.spicedb import SpiceDBClient

    respx_mock.post("http://spicedb.example.com/v1/permissions/check").mock(
        return_value=httpx.Response(
            200,
            json={
                "permissionship": "PERMISSIONSHIP_HAS_PERMISSION",
                "checkedAt": {"token": "tok123"},
            },
        )
    )
    async with httpx.AsyncClient() as client:
        spice = SpiceDBClient(
            base_url="http://spicedb.example.com",
            token="secret",
            client=client,
        )
        result = await spice.check("document", "doc1", "read", "user", "u1")
    assert result.allowed is True
    assert result.zookie == "tok123"


async def test_spicedb_check_denied(respx_mock):
    import httpx
    from eel_auth.spicedb import SpiceDBClient

    respx_mock.post("http://spicedb.example.com/v1/permissions/check").mock(
        return_value=httpx.Response(
            200,
            json={"permissionship": "PERMISSIONSHIP_NO_PERMISSION"},
        )
    )
    async with httpx.AsyncClient() as client:
        spice = SpiceDBClient(
            base_url="http://spicedb.example.com",
            token="secret",
            client=client,
        )
        result = await spice.check("document", "doc1", "write", "user", "u1")
    assert result.allowed is False
