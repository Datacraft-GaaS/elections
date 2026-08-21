# Policy: transparency-svc
# Grants minimum required access for transparency-svc workload.
# Extra: STH signing for transparency log and code-signing for release artifacts.

path "kv/data/eel/+/transparency-svc/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/+/transparency-svc/*" {
  capabilities = ["list", "read"]
}

path "transit/eel/encrypt/transparency-svc-data" {
  capabilities = ["update"]
}
path "transit/eel/decrypt/transparency-svc-data" {
  capabilities = ["update"]
}

# Signed tree head (STH) signing for the append-only transparency log.
path "transit/eel/sign/sth-signing" {
  capabilities = ["update"]
}
path "transit/eel/verify/sth-signing" {
  capabilities = ["update"]
}

# Code signing for artifact publication.
path "transit/eel/sign/code-signing" {
  capabilities = ["update"]
}
path "transit/eel/verify/code-signing" {
  capabilities = ["update"]
}

path "database/eel/creds/transparency-svc-role" {
  capabilities = ["read"]
}

path "pki/eel/intermediate/issue/transparency-svc-role" {
  capabilities = ["update"]
}
