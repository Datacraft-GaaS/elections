# Policy: training-svc
# Grants minimum required access for training-svc workload.

path "kv/data/eel/+/training-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/training-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/training-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/training-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/training-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/training-svc-role" {
  capabilities = ["update"]
}
