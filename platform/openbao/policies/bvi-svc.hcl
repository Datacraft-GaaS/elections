# Policy: bvi-svc
# Grants minimum required access for bvi-svc workload.

path "kv/data/eel/+/bvi-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/bvi-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/bvi-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/bvi-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/bvi-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/bvi-svc-role" {
  capabilities = ["update"]
}
