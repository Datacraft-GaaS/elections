# Keycloak realm upgrade (blue-green)

## Context

Keycloak realm JSON upgrades must be rehearsed on staging before production. Importing a broken realm to production can lock out all service clients for the affected trust domain. This runbook covers a blue-green approach: import the new configuration to a parallel staging realm with a `-green` suffix, run golden tests, then cut over traffic. The old realm is not deleted until all sessions have expired and traffic is confirmed stable on the new realm.

The `iam-seed` binary (`./target/release/iam-seed`) is the authoritative tool for applying realm JSON. Do not use the Keycloak Admin UI for schema changes; UI changes are not reproducible and bypass git history.

## Pre-conditions

Before beginning:

- The new realm JSON is committed to `platform/keycloak/realms/<realm>.json` in git. No uncommitted local modifications.
- `./target/release/iam-seed verify --env dev` passes against the dev Keycloak instance.
- Staging Keycloak is accessible at `https://keycloak.eel.staging` and the staging OpenBao instance holds a valid admin token at `kv/eel/staging/keycloak/admin-token`.
- The `iam-seed` binary is built from the current commit: `cargo build --release -p iam-seed`.
- An ITSM change ticket is open for the realm upgrade.

## Procedure

### Step 1: validate JSON syntax

Catch syntax errors before touching any Keycloak instance:

```bash
python3 -c "import json, sys; json.load(sys.stdin)" \
  < platform/keycloak/realms/<realm>.json
echo "JSON valid"
```

If `json.load` raises, fix the syntax error before continuing.

### Step 2: import to staging realm with -green suffix

```bash
export KC_URL="https://keycloak.eel.staging"
export KC_ADMIN_TOKEN="$(bao kv get -field=value kv/eel/staging/keycloak/admin-token)"

./target/release/iam-seed keycloak-apply \
  --realm <realm-name>-green \
  --url "$KC_URL" \
  --token "$KC_ADMIN_TOKEN" \
  --realm-file platform/keycloak/realms/<realm>.json \
  --yes
```

The `--yes` flag suppresses the interactive confirmation prompt. Confirm the realm appears in Keycloak:

```bash
curl -s -H "Authorization: Bearer $KC_ADMIN_TOKEN" \
  "$KC_URL/admin/realms" \
  | python3 -c "import json,sys; [print(r['realm']) for r in json.load(sys.stdin)]" \
  | grep <realm-name>-green
```

### Step 3: run golden authz tests against staging

```bash
export SPICEDB_ENDPOINT="spicedb.eel.staging:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/staging/spicedb/token)"

./target/release/iam-seed verify --env staging
```

All assertions must pass. If any fail, do not proceed to production. Investigate the diff between the new realm JSON and the expected state.

### Step 4: smoke test OIDC token issuance from the green realm

```bash
curl -s -X POST \
  "$KC_URL/realms/<realm-name>-green/protocol/openid-connect/token" \
  -d "client_id=iam-seed-test" \
  -d "client_secret=<iam-seed-test-client-secret>" \
  -d "grant_type=client_credentials" \
  | python3 -c "import json,sys; t=json.load(sys.stdin); print('token_type:', t.get('token_type')); print('error:', t.get('error','none'))"
```

Expected output: `token_type: Bearer`, `error: none`. Any error indicates a misconfigured client or realm.

Also confirm the JWKS endpoint returns keys:

```bash
curl -s "$KC_URL/realms/<realm-name>-green/protocol/openid-connect/certs" \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print('key count:', len(d['keys']))"
```

Expect at least one key.

### Step 5: cut over to the new realm on staging

Keycloak does not support in-place realm rename. The cut-over is done by applying the realm JSON to the canonical realm name on staging, which overwrites the existing configuration:

```bash
./target/release/iam-seed keycloak-apply \
  --realm <realm-name> \
  --url "$KC_URL" \
  --token "$KC_ADMIN_TOKEN" \
  --realm-file platform/keycloak/realms/<realm>.json \
  --yes
```

Update any ArgoCD `Application` resources or Kubernetes `ConfigMap` references that point to the old realm name to point to `<realm-name>`.

### Step 6: apply to production

Only after staging cut-over is confirmed stable. Use the production OpenBao instance:

```bash
export KC_URL_PROD="https://keycloak.eel.internal"
export KC_ADMIN_TOKEN_PROD="$(bao kv get -field=value kv/eel/prod/keycloak/admin-token)"

./target/release/iam-seed keycloak-apply \
  --realm <realm-name> \
  --url "$KC_URL_PROD" \
  --token "$KC_ADMIN_TOKEN_PROD" \
  --realm-file platform/keycloak/realms/<realm>.json \
  --yes
```

Run golden tests against production immediately after:

```bash
export SPICEDB_ENDPOINT="spicedb.eel.prod:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/prod/spicedb/token)"

./target/release/iam-seed verify --env prod
```

### Step 7: decommission the green staging realm

Wait for all active sessions in the green realm to expire. The SSO session max for this realm is 8 hours. After that window:

```bash
# Confirm no active sessions remain
curl -s -H "Authorization: Bearer $KC_ADMIN_TOKEN" \
  "$KC_URL/admin/realms/<realm-name>-green/sessions/stats" \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print('active sessions:', d)"

# Delete the green realm
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh delete realms/<realm-name>-green
```

## Rollback

If the new realm configuration causes failures on staging or production, re-apply the previous realm JSON from git:

```bash
git show HEAD~1:platform/keycloak/realms/<realm>.json > /tmp/<realm>-prev.json
./target/release/iam-seed keycloak-apply \
  --realm <realm-name> \
  --url "$KC_URL_PROD" \
  --token "$KC_ADMIN_TOKEN_PROD" \
  --realm-file /tmp/<realm>-prev.json \
  --yes
```

Update ArgoCD to point back to the previous commit. Open a follow-up ticket to diagnose why the new realm JSON failed before attempting the upgrade again.

## Post-upgrade checklist

Confirm all of the following before closing the change ticket:

- JWKS endpoint on the production realm returns keys: `curl -s https://keycloak.eel.internal/realms/<realm-name>/protocol/openid-connect/certs | python3 -c "import json,sys; print(len(json.load(sys.stdin)['keys']), 'keys')"`.
- All service clients can authenticate: run `./target/release/iam-seed verify --env prod` and confirm all checks pass.
- Integration tests pass: `uv run pytest -vxs tests/ci -k keycloak`.
- No error spikes in the Keycloak `keycloak-events` topic for the realm in question.
- Change ticket updated with git commit hash of the realm JSON applied.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-pod-kill-boundary-svc.yaml`

The boundary-svc kill scenario verifies that a realm upgrade does not lock out service clients when the API gateway restarts during a promotion window.
