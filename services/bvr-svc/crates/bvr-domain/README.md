# bvr-domain

Voter aggregates, biometric templates, audit events, and merkle tree for register integrity.

Defines Voter aggregate (versioned, event-sourced with full audit history), BiometricTemplate (fingerprint + face encoding), AuditEvent (enrolled, updated, deleted, inspected, locked), MerkleRoot (chain hash for register integrity proof). Implements domain logic: voter dedup checks, data minimisation (GDPR/DPA principles), audit trail immutability.

Fits into bvr-svc as the domain model: encodes voter register business rules and Kenya DPA 2019 compliance.

## Public API

Exposed modules via lib.rs:
- voter: Voter, apply_voter_change(change) -> UpdatedVoter
- biometric: BiometricTemplate, create_from_capture(fingerprint, face), deduplicate_against(other)
- audit: AuditEvent, log_voter_event(voter_id, event_type, reason), get_audit_trail(voter_id)
- merkle: MerkleRoot, update_register_root(new_voters, deleted_voters), verify_integrity()

## Feature flags

None currently.

## Testing

Unit tests (domain logic):
```bash
cargo test --lib
```

Audit trail immutability:
```bash
cargo test audit::immutability
```

Merkle tree consistency:
```bash
cargo test merkle::consistency
```

Property tests (voter versioning):
```bash
cargo test --lib voter::properties
```

Integration tests:
```bash
cargo nextest run
```

## Dependencies of note

Workspace:
- eel-eventstore-rs: Event persistence
- eel-events-rs: Event schema
- eel-crypto-rs: Merkle hashing

External:
- serde: Voter serialization
- sha2: Merkle hashing

## Related docs

See services/bvr-svc/docs/voter-aggregate.md for voter structure.
See services/bvr-svc/docs/biometric-template.md for template format.
See services/bvr-svc/docs/audit-trail-design.md for audit semantics.
See services/bvr-svc/docs/register-merkle-tree.md for integrity proofs.
