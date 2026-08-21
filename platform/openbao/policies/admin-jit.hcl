# Policy: admin-jit
# Just-in-time admin credentials with a 4-hour TTL cap.
# Attach this policy to a token role that enforces token_max_ttl = "4h".
# Grants broad read access for incident response, but no write to secret data.

# Read all KV secret data across all eel environments.
path "kv/data/eel/*" {
  capabilities = ["read"]
}
path "kv/metadata/eel/*" {
  capabilities = ["list", "read"]
}

# Issue TLS certificates for any eel service (needed for mTLS debugging).
path "pki/eel/intermediate/issue/*" {
  capabilities = ["update"]
}

# Lease management: list, look up, and renew leases.
path "sys/leases/lookup" {
  capabilities = ["update"]
}
path "sys/leases/renew" {
  capabilities = ["update"]
}
path "sys/leases/revoke" {
  capabilities = ["update"]
}
path "sys/leases/revoke-prefix/*" {
  capabilities = ["update"]
}

# Token creation with an enforced max TTL of 4 hours.
# The token role must set token_max_ttl = "4h" to enforce this bound.
path "auth/token/create" {
  capabilities = ["update"]
}
path "auth/token/create/*" {
  capabilities = ["update"]
}
path "auth/token/lookup" {
  capabilities = ["update"]
}
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Read-only view of mounted secrets engines and auth methods.
path "sys/mounts" {
  capabilities = ["read"]
}
path "sys/auth" {
  capabilities = ["read"]
}
path "sys/policies/acl" {
  capabilities = ["list", "read"]
}
path "sys/policies/acl/*" {
  capabilities = ["read"]
}
