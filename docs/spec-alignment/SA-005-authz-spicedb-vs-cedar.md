# SA-005: Authorization engine: SpiceDB (ReBAC/Zanzibar) vs Cedar

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/the Commission/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §14.4 pins **Cedar** (Amazon, Apache 2.0) as the fine-grained authorization engine, alongside OPA for admission policy.

## Decision (per ADRs)

**SpiceDB** (Authzed, Apache 2.0) for domain-resource authorization. Retain **OPA + Kyverno** for admission and cluster policy (no conflict, different concerns).

## RFP status

**Not RFP-binding.** tender.txt L2626-2628 mandates RBAC + MFA with method open. No specific authz engine named. Confirmed by grep.

## Rationale

1. **Election permissions are inherently relational.** Real examples from the module set:
   - "An observer with active accreditation A can read constituency results for C only if C is in A's accreditation set AND the current time is past close-of-poll for C."
   - "A party agent for party P can attest to Form 34A for polling station S only if P has a candidate on the ballot at S AND agent A is the currently-delegated agent for S."
   - "A media accreditee can access the results firehose only if their accreditation status is active AND their credential has not been revoked in the last 60 seconds."
   These are all **relationship traversals over live state**, Zanzibar's home turf.
2. **SpiceDB's zookies (consistency tokens)** enforce read-after-write consistency across the authz decision graph. Under adversarial pressure this matters: a permission revocation must be visible to every service within a bounded time, and a request whose authz was decided against stale state must fail-close.
3. **Cedar can express relational permissions**, but does so verbosely and requires the calling service to hand it the full context. SpiceDB stores the relationships once, so any service asking `CheckPermission(user, action, resource)` gets the same answer without re-shipping context.
4. **Operational history**. SpiceDB has 3+ years of production deployment at scale (Authzed, Netflix, Turo, Datadog). Cedar is younger in the deployed-at-scale category.
5. **Client-fleet operational familiarity**. The client's existing infrastructure already runs SpiceDB. Operational muscle exists.
6. **Ecosystem**. SpiceDB has a mature schema language (`.zed`), CLI (`zed`), migration tooling, and testing framework (`spicedb-testcontainer`). Cedar is on par for the policy layer but weaker on the relationship-store side.

## Trade-offs

- Cedar's DSL is more approachable for policy authors coming from IAM background; SpiceDB's schema requires learning the Zanzibar mental model (about 1-2 weeks for a competent engineer).
- Cedar policies are pure functions (no external state); SpiceDB is a stateful service (one more datastore to operate + backup + fail-over). Real cost; mitigated by SpiceDB being architecturally simple (Postgres-backed).
- Cedar has AWS-native ergonomics (IAM Identity Center integration); SpiceDB is provider-agnostic.

## Edits to apply to sw/

If Cedar is mandated: use **Cedar for pure policy expressions** (approval workflows, config predicates) + **SpiceDB or a custom relationship store on Postgres** for the relational-permission cases. Two engines is worse than one, but the relational cases cannot be expressed cleanly in Cedar alone without shipping every relationship as call-time context.

## Related

ADR-0007 (authn/authz/secrets), ADR-0023 (RFP-alignment).
