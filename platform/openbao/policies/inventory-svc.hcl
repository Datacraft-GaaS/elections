# Policy: inventory-svc
# Grants minimum required access for inventory-svc workload.

path "kv/data/eel/+/inventory-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/inventory-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/inventory-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/inventory-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/inventory-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/inventory-svc-role" {
  capabilities = ["update"]
}
