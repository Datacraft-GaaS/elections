# hsm-ceremony

CLI for IEBC custodians to manage eel HSM key ceremonies.

## Prerequisites

- Access to the HSM signer gRPC endpoint (default `http://localhost:8443`).
- An Ed25519 signing key (32-byte seed file) for each custodian.
- `EEL_HSM_SIGNER_ENDPOINT` set to the production endpoint.

## Full workflow example (break-glass runbook)

### 1. Register custodians (done once at ceremony setup)

```sh
# Generate a custodian key pair (example using openssl or any Ed25519 tool)
openssl genpkey -algorithm Ed25519 -out alice.pem
openssl pkey -in alice.pem -pubout -out alice.pub.pem
# Export raw 32-byte pubkey
openssl pkey -in alice.pub.pem -pubin -outform DER | tail -c 32 > alice.pub.raw

hsm-ceremony custodian register --name alice --pubkey alice.pub.raw
# Output includes custodian_id; save it.

# Repeat for bob, carol, dave, eve.
```

### 2. Request a key rotation ceremony

```sh
hsm-ceremony ceremony request \
  --role cycle-kek \
  --purpose "2027 election cycle KEK rotation" \
  --quorum-m 3 --quorum-n 5
# Output: ceremony_id, operation_hash. Distribute these to custodians out-of-band.
```

### 3. Each custodian approves independently

```sh
# Export raw 32-byte seed from Alice's key:
openssl pkey -in alice.pem -outform DER | tail -c 32 > alice.seed.raw

hsm-ceremony ceremony approve \
  --ceremony-id <ceremony_id> \
  --custodian-key alice.seed.raw \
  --custodian-id <alice_custodian_id> \
  --operation-hash <operation_hash> \
  --format json
# Prints signature_hex; submit to HSM signer operator.
```

### 4. Check status

```sh
hsm-ceremony ceremony status --ceremony-id <ceremony_id>
```

### 5. Abort if needed (any custodian, while Requested or AwaitingQuorum)

```sh
hsm-ceremony ceremony abort \
  --ceremony-id <ceremony_id> \
  --reason "suspected key material compromise"
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `EEL_HSM_SIGNER_ENDPOINT` | `http://localhost:8443` | gRPC endpoint |
| `EEL_HSM_PIN` | `1234` | PKCS#11 user PIN (server-side) |
| `EEL_HSM_VENDOR` | `softhsm` | HSM vendor: softhsm, luna7, utimaco |

## JSON output

All subcommands accept `--format json` for machine-readable output suitable for
scripting or piping into `jq`.
