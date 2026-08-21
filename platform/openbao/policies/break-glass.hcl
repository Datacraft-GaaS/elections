# Policy: break-glass
# Shamir custodian quorum path for emergency root operations.
# This policy is attached to tokens generated only during a declared emergency
# requiring a quorum of Shamir key holders.
# Custodians may NOT read or write normal operational secrets.

# Shamir unseal operations.
path "sys/unseal" {
  capabilities = ["update"]
}
path "sys/seal-status" {
  capabilities = ["read"]
}

# Root token generation (requires quorum nonce exchange).
path "auth/token/root-generation/*" {
  capabilities = ["create", "read", "update", "delete"]
}
path "sys/generate-root" {
  capabilities = ["read", "update"]
}
path "sys/generate-root/*" {
  capabilities = ["create", "read", "update", "delete"]
}

# Shamir rekey (rotate unseal key shares).
path "sys/rekey/*" {
  capabilities = ["create", "read", "update", "delete"]
}
path "sys/rekey-recovery-key/*" {
  capabilities = ["create", "read", "update", "delete"]
}

# Read seal/init status only.
path "sys/init" {
  capabilities = ["read"]
}

# Explicit deny: break-glass custodians must not touch operational secrets.
# This deny overrides any other policy that might grant access.
path "kv/*" {
  capabilities = ["deny"]
}
path "database/*" {
  capabilities = ["deny"]
}
path "transit/*" {
  capabilities = ["deny"]
}
path "pki/*" {
  capabilities = ["deny"]
}
path "auth/token/create" {
  capabilities = ["deny"]
}
