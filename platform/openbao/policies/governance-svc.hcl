# Policy: governance-svc
# Grants minimum required access for governance-svc workload.

path "kv/data/eel/+/governance-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/governance-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/governance-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/governance-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/governance-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/governance-svc-role" {
  capabilities = ["update"]
}
