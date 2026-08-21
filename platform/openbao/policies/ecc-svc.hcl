# Policy: ecc-svc
# Grants minimum required access for ecc-svc workload.

path "kv/data/eel/+/ecc-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/ecc-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/ecc-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/ecc-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/ecc-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/ecc-svc-role" {
  capabilities = ["update"]
}
