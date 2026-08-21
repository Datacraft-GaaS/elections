# Architecture overview

## Shape

Eighteen Kubernetes services implement sixty-nine modules (ADR-0020). Every service is built from the same shared packages and deployed identically to two IEBC-provided DR sites and five regional edge clusters (ADR-0019).

```
kits (Flutter)   mobile (Flutter)   portals (Astro / LiveView / Django)   desktop (Tauri)
      \                 |                     |                                /
       +----------------+----- edge: Envoy Gateway, Coraza, Varnish, Anubis ---+
                                |
              services/<svc>  (Rust default; Python for ML and vendor SDKs; Elixir for LiveView)
                                |
        packages/eel-*  (auth, secrets, events, outbox, eventstore, pii, hlc, resilience, observability)
                                |
   PostgreSQL 17 (T1 voter / consensus / audit, T2 shared)   Kafka + Karapace + Debezium   Valkey   MinIO   OpenSearch
                                |
   Keycloak + SPIFFE/SPIRE + SpiceDB + OpenBao + OPA/Kyverno        HSM (PKCS#11) + transparency log (transparency-svc)
```

## Rules

1. Business logic lives in `services/<svc>/`; portals render, admin apps administer, kits and mobile apps consume the public API (ADR-0011). No parallel implementations of a rule.
2. Services own their data. No cross-schema joins, no shared writes (ADR-0012). Cross-service state moves only through the transactional outbox onto Kafka (ADR-0015).
3. Only four bounded contexts are event-sourced: rts (M08/09/10, M51, M63), custody (M20). Everything else is state plus an immutable audit shadow (ADR-0013).
4. Every consensus-relevant artefact is signed and provable against the transparency log (ADR-0017); every write endpoint is idempotent; every service ships degradation modes (ADR-0009).
5. No identifier carries the client name. Packages, events, topics, headers and trust domains use the `eel` prefix.

## Where to read next

`docs/adr/README.md` for decisions, `docs/plans/README.md` for the build plan, `docs/modules/README.md` for the module index, `docs/EVENT-CONVENTIONS.md` for event authoring.
