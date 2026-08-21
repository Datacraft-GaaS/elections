# Policy: biometric-edge-svc
# Grants minimum required access for biometric-edge-svc workload.

path "kv/data/eel/+/biometric-edge-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/biometric-edge-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/biometric-edge-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/biometric-edge-svc-data" {
  capabilities = ["update"]
}

path "database/eel/creds/biometric-edge-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/biometric-edge-svc-role" {
  capabilities = ["update"]
}
