# Policy: boundary-svc
# Grants minimum required access for boundary-svc workload.

path "kv/data/eel/+/boundary-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/boundary-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/boundary-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/boundary-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/boundary-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/boundary-svc-role" {
  capabilities = ["update"]
}
