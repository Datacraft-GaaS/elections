package eel.authz.decisions

import future.keywords.if
import future.keywords.in

# Generic ADR-0007 authz check pattern.
# Services may layer Rego decisions on top of SpiceDB relationship checks.
# Input shape:
#   input.subject  - SPIFFE SVID or user ID
#   input.action   - verb (read, write, delete, …)
#   input.resource - resource URI (e.g. "rts://result/KE-001-001")
#   input.context  - map of additional context (tier, election_phase, …)
#   input.spicedb  - result of the SpiceDB lookup already performed by the service

default allow = false

# Allow if SpiceDB said yes and no Rego override denies
allow if {
	input.spicedb.permitted == true
	not deny
}

# Deny if in election_day_mode and action is not read-only
deny if {
	input.context.election_phase == "election_day_mode"
	input.action != "read"
	input.action != "verify"
	input.action != "audit"
}

# Deny writes to T1 resources from non-T1 SVIDs
deny if {
	is_t1_resource(input.resource)
	input.action in {"write", "delete", "admin"}
	not is_t1_svid(input.subject)
}

# Deny access if subject SVID trust domain does not match environment
deny if {
	not valid_trust_domain(input.subject)
}

# ---------- helpers ----------

is_t1_resource(r) if startswith(r, "rts://")
is_t1_resource(r) if startswith(r, "bvr://")
is_t1_resource(r) if startswith(r, "abis://")

is_t1_svid(s) if startswith(s, "spiffe://eel.prod/ns/rts-svc/")
is_t1_svid(s) if startswith(s, "spiffe://eel.prod/ns/bvr-svc/")
is_t1_svid(s) if startswith(s, "spiffe://eel.prod/ns/abis-svc/")
is_t1_svid(s) if startswith(s, "spiffe://eel.prod/ns/transparency-svc/")

valid_trust_domain(s) if contains(s, "spiffe://eel.")
