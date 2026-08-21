# Policy: vprs-svc
# Grants minimum required access for vprs-svc workload.

path "kv/data/eel/+/vprs-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/vprs-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/vprs-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/vprs-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/vprs-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/vprs-svc-role" {
  capabilities = ["update"]
}
