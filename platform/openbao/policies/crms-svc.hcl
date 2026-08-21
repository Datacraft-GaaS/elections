# Policy: crms-svc
# Grants minimum required access for crms-svc workload.

path "kv/data/eel/+/crms-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/crms-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/crms-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/crms-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/crms-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/crms-svc-role" {
  capabilities = ["update"]
}
