# declaration

Module M51 election declaration and certificate of election with HSM signing and witness tracking.

Issues digital Declaration and CertificateOfElection from tallied results. Signs via HSM (Ed25519 + ML-DSA). Tracks witness cosignatures (Chairperson, IEBC Secretary, observers). Manages petition window (7-day deadline post-declaration). Emits rts.declaration.signed on signature completion.

Fits into rts-svc as the electoral authority signing path: RTS tally flows into declaration issuance, HSM signs, transparency log publishes.

## Public API

Exposed modules via lib.rs:
- declaration: Declaration, issue_from_tally(tally_snapshot) -> DeclarationDraft
- certificate: CertificateOfElection, sign_with_hsm(draft) -> SignedCertificate
- witness: WitnessSignature, collect_witness_sig(witness_id, sig) -> WitnessTracker
- petition: PetitionWindow, open_challenge_period() -> PetitionDeadline

## Feature flags

None currently.

## Testing

Unit tests (declaration logic):
```bash
cargo test --lib
```

Certificate signature verification:
```bash
cargo test certificate::verify
```

Witness tracking tests:
```bash
cargo test witness::
```

Integration tests (HSM):
```bash
cargo nextest run --features test-hsm
```

## Dependencies of note

Workspace:
- hsm-signer: HSM signing operations
- rts-domain: Tally snapshot integration
- eel-observability-rs: Signing audit trails

External:
- ed25519-dalek: Signature verification
- tokio: Async HSM calls

## Related docs

See services/rts-svc/docs/declaration-schema.md for format spec.
See services/rts-svc/docs/witness-cosigning.md for witness procedures.
See docs/runbooks/declaration-hsm-unavailable.md for fallback procedures.
