# Policy: ai-platform-svc
# Grants minimum required access for ai-platform-svc workload.

path "kv/data/eel/+/ai-platform-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/ai-platform-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/ai-platform-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/ai-platform-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/ai-platform-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/ai-platform-svc-role" {
  capabilities = ["update"]
}
