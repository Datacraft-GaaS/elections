# Policy: abis-svc
# Grants minimum required access for abis-svc workload.
# Extra: decrypt access to abis-biometric-data for biometric record processing.

path "kv/data/eel/+/abis-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/abis-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/abis-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/abis-svc-data" {
  capabilities = ["update"]
}

# Biometric payload decryption (abis-svc owns this key).
path "transit/eel/decrypt/abis-biometric-data" {
  capabilities = ["update"]
}

path "database/eel/creds/abis-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/abis-svc-role" {
  capabilities = ["update"]
}
