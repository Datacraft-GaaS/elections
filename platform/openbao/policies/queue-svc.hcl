# Policy: queue-svc
# Grants minimum required access for queue-svc workload.

path "kv/data/eel/+/queue-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/queue-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/queue-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/queue-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/queue-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/queue-svc-role" {
  capabilities = ["update"]
}
