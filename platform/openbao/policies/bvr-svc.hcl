# Policy: bvr-svc
# Grants minimum required access for bvr-svc workload.

path "kv/data/eel/+/bvr-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/bvr-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/bvr-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/bvr-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/bvr-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/bvr-svc-role" {
  capabilities = ["update"]
}
