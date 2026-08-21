# Policy: field-mobile-svc
# Grants minimum required access for field-mobile-svc workload.

path "kv/data/eel/+/field-mobile-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/field-mobile-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/field-mobile-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/field-mobile-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/field-mobile-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/field-mobile-svc-role" {
  capabilities = ["update"]
}
