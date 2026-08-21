# Policy: support-admin-svc
# Grants minimum required access for support-admin-svc workload.

path "kv/data/eel/+/support-admin-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/support-admin-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/support-admin-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/support-admin-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/support-admin-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/support-admin-svc-role" {
  capabilities = ["update"]
}
