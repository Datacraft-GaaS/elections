# Policy: portal-svc
# Grants minimum required access for portal-svc workload.

path "kv/data/eel/+/portal-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/portal-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/portal-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/portal-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/portal-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/portal-svc-role" {
  capabilities = ["update"]
}
