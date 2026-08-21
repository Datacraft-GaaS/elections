# Break-glass procedure

## What break-glass is

The break-glass procedure provides last-resort access to root credentials across Keycloak, OpenBao, SpiceDB, and HSM when normal operator access is unavailable. Triggering conditions include catastrophic credential loss, key personnel unavailability, and suspected insider threat requiring full credential rotation. Every action taken during a break-glass event is recorded to the transparency log via `transparency-svc` gRPC calls. This procedure is designed for use under two-person integrity at all times.

## When to invoke

Invoke only when all of the following are true:

- A specific triggering condition exists: confirmed credential compromise, catastrophic loss of access to production auth infrastructure, or an insider threat requiring immediate full rotation.
- A quorum of at least 3 of the 5 custodians has agreed out-of-band that invocation is warranted.
- An incident ticket is open in the ITSM system. The ticket number must be recorded in every transparency log leaf emitted during this procedure. Never invoke without an open ticket.

## Custodians

Five custodians hold Shamir key shares. Quorum is any 3 of 5. No single organization holds a majority.

| Role | Organization |
|---|---|
| Chair representative | Chair |
| CJ representative | CJ |
| AG representative | AG |
| Operational alternate 1 | Operations |
| Operational alternate 2 | Operations |

A single organization contributes at most two custodians, so no organization can form quorum alone.

## What it unlocks

- Keycloak root admin: a temporary admin user created in the master realm using the stored root admin password retrieved from OpenBao.
- OpenBao root token: reconstructed via Shamir secret sharing from 5 key shares (threshold 3-of-5). The root token is single-use and must be revoked immediately after the operation it was needed for.
- SpiceDB admin API key: stored encrypted in OpenBao at `kv/eel/prod/spicedb/admin-api-key`. Retrieved once OpenBao is unsealed.
- HSM custodian shares: M-of-N (3-of-5) physical ceremony. Requires presence at the HSM. See `docs/runbooks/hsm-key-ceremony.md`.

## Pre-conditions

Before beginning:

- Establish a secure out-of-band channel between all participating custodians (Signal group call or in-person).
- Open an incident ticket and record the ticket number. Example: `INC-2025-0042`.
- Where possible, perform all operations from a clean, air-gapped workstation with no persistent browser sessions or credential caches.
- Maintain two-person integrity throughout: no custodian acts alone on any step. A second custodian must observe and confirm each command.
- Screen-record or maintain a written log of every command run.

## Procedure

### Step 1: unseal OpenBao using Shamir shares

Each custodian SSHs to the OpenBao primary server (`openbao.eel.internal`) and provides their share. Shares are prompted interactively and not echoed.

```bash
export OPENBAO_ADDR="https://openbao.eel.internal:8200"
bao operator unseal
```

Repeat for each custodian until the threshold (3 shares) is met. Verify the sealed status drops to false:

```bash
bao status | grep -E "Sealed|Threshold|Shares"
```

Expected output shows `Sealed: false`, `Threshold: 3`, `Shares: 5`.

Emit transparency log leaf:

```bash
grpcurl -plaintext \
  -d '{"event_type":"meta-events.break-glass.unseal.v1","payload":{"incident_ticket":"INC-2025-0042","step":"openbao-unseal","custodian_count":3}}' \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 2: reconstruct OpenBao root token (only if root-level operations are required)

A root token is needed only if the JIT admin policy does not cover the required operations. If a scoped token suffices, skip this step.

```bash
bao operator generate-root -init
```

Save the `Nonce` and `OTP` from the output. Then each custodian provides their unseal key share:

```bash
bao operator generate-root -nonce <NONCE>
```

Repeat for 3 custodians. After the third share, decode the encoded root token:

```bash
bao operator generate-root -decode <ENCODED_TOKEN> -otp <OTP>
```

The decoded value is the root token. Use it immediately for required operations, then revoke:

```bash
bao token revoke <ROOT_TOKEN>
```

Emit tlog leaf:

```bash
grpcurl -plaintext \
  -d '{"event_type":"meta-events.break-glass.root-token.v1","payload":{"incident_ticket":"INC-2025-0042","step":"openbao-root-token","action":"generated-and-revoked"}}' \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 3: retrieve SpiceDB admin API key

With OpenBao unsealed, retrieve the SpiceDB admin key. Use a JIT admin token if available, or the root token from step 2 as a last resort.

```bash
export OPENBAO_ADDR="https://openbao.eel.internal:8200"
export OPENBAO_TOKEN="<JIT_ADMIN_TOKEN_OR_ROOT_TOKEN>"
bao kv get kv/eel/prod/spicedb/admin-api-key
```

Record the key in the secure channel only. Do not write it to disk or paste it into chat.

Emit tlog leaf:

```bash
grpcurl -plaintext \
  -d '{"event_type":"meta-events.break-glass.spicedb-key.v1","payload":{"incident_ticket":"INC-2025-0042","step":"spicedb-key-retrieval","actor":"<CUSTODIAN_ID>"}}' \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 4: recover Keycloak root admin

Retrieve the Keycloak root admin password from OpenBao and create a temporary break-glass admin account in the master realm.

```bash
export OPENBAO_ADDR="https://openbao.eel.internal:8200"
export OPENBAO_TOKEN="<JIT_ADMIN_TOKEN_OR_ROOT_TOKEN>"

# Authenticate kcadm.sh against Keycloak master realm
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh config credentials \
    --server https://keycloak.eel.internal \
    --realm master \
    --user admin \
    --password "$(bao kv get -field=value kv/eel/prod/keycloak/root-admin-password)"

# Create the temporary break-glass admin user
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh create users -r master \
    -s username=break-glass-admin \
    -s enabled=true \
    -s 'credentials=[{"type":"password","value":"<TEMP_PASSWORD>","temporary":true}]'

# Assign realm admin role
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh add-roles -r master \
    --uusername break-glass-admin --rolename admin
```

Choose `<TEMP_PASSWORD>` as a high-entropy value (32+ chars). Record it only in the secure channel. The `temporary: true` flag forces a password change on first login.

Emit tlog leaf:

```bash
grpcurl -plaintext \
  -d '{"event_type":"meta-events.break-glass.keycloak-admin.v1","payload":{"incident_ticket":"INC-2025-0042","step":"keycloak-break-glass-user-created","username":"break-glass-admin"}}' \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 5: HSM custodian ceremony (if HSM root key recovery is required)

If the incident requires HSM root key material, follow `docs/runbooks/hsm-key-ceremony.md`. The ceremony requires physical presence at the HSM and uses the same 3-of-5 custodian set. Do not attempt the HSM ceremony remotely.

## Verification

After each step, confirm the recovered credential grants the expected access before proceeding.

Keycloak: list realms to confirm master realm admin works:

```bash
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh get realms --fields realm,enabled
```

OpenBao: list the secrets mount to confirm policy coverage:

```bash
bao kv list kv/eel/prod/
```

SpiceDB: confirm the admin API key reaches the endpoint:

```bash
export SPICEDB_ENDPOINT="spicedb.eel.prod:50051"
./tools/spicedb/zed schema read \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "<RETRIEVED_ADMIN_API_KEY>" \
  | head -5
```

## Rotation post-incident

All credentials touched during break-glass must be rotated before the incident is closed.

Rotate Keycloak root admin password (generate a new one and write it back to OpenBao):

```bash
bao kv put kv/eel/prod/keycloak/root-admin-password value="<NEW_PASSWORD>"
```

Revoke the break-glass admin user in Keycloak:

```bash
kubectl exec -n keycloak deploy/keycloak -- \
  /opt/keycloak/bin/kcadm.sh delete users/<BREAK_GLASS_USER_ID> -r master
```

Revoke the OpenBao root token if it was generated and not yet revoked:

```bash
bao token revoke <ROOT_TOKEN>
```

Re-seal OpenBao:

```bash
bao operator seal
```

Rotate Shamir shares so that the shares used during this event are no longer valid:

```bash
bao operator rekey -init -key-shares=5 -key-threshold=3
# Each custodian provides their current share:
bao operator rekey -nonce <REKEY_NONCE>
# Distribute new shares to custodians via secure channel
```

Rotate the SpiceDB admin API key:

```bash
# Generate a new key and write to OpenBao
NEW_KEY=$(openssl rand -hex 32)
bao kv put kv/eel/prod/spicedb/admin-api-key value="$NEW_KEY"
# Update SpiceDB server configuration to accept new key, then restart
```

Each rotation must emit a tlog leaf. Template:

```bash
grpcurl -plaintext \
  -d '{"event_type":"meta-events.break-glass.rotation.v1","payload":{"incident_ticket":"INC-2025-0042","step":"<ROTATION_STEP>","credential":"<CREDENTIAL_NAME>"}}' \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

Replace `<ROTATION_STEP>` with a descriptive label such as `keycloak-root-password`, `openbao-shares`, `spicedb-admin-key`.

## Transparency log leaf template

Every step must emit a leaf before moving to the next step. The general template:

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.break-glass.<STEP>.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"step\":\"<STEP_LABEL>\",\"actor\":\"<CUSTODIAN_ID>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

Do not proceed to the next step if the `grpcurl` call fails. Investigate connectivity to `transparency-svc` before continuing.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-hsm-unavailable.yaml` and `chaos/scenarios/eel-spicedb-down.yaml`

Run under `docs/runbooks/chaos-monthly-drill.md` supervision. The HSM scenario covers the Shamir share assembly steps; the SpiceDB down scenario covers the credential rotation steps.
