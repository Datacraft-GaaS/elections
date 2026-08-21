# Policy: rts-svc
# Grants minimum required access for rts-svc workload.
# Extra: sign voter declarations via transit/eel/sign/declaration-signing.

path "kv/data/eel/+/rts-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/rts-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/rts-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/rts-svc-data" {
  capabilities = ["update"]
}

# Declaration signing for voter result transmissions.
path "transit/eel/sign/declaration-signing" {
  capabilities = ["update"]
}
path "transit/eel/verify/declaration-signing" {
  capabilities = ["update"]
}

path "database/eel/creds/rts-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/rts-svc-role" {
  capabilities = ["update"]
}
