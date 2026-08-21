# Architecture overview

## Shape

Three shells (desktop, mobile, server) consume the same 30 feature modules. Each module owns a vertical slice: domain model, server implementation, and platform bindings.

```
+-------------+      +-------------+      +-------------+
|  desktop    |      |   mobile    |      |   server    |
+------+------+      +------+------+      +------+------+
       |                    |                    |
       +---------+----------+----------+---------+
                 |                     |
           packages/sdk         packages/shared
                 |                     |
           +-----+---------------------+-----+
           |                                 |
           |             modules/*           |
           |   shared / server / desktop /   |
           |   mobile / docs                 |
           +---------------------------------+
```

## Rules

1. `modules/*/shared` is pure: no I/O, no framework imports. Everything else in a module depends on it, never the reverse.
2. Cross-module calls go through documented contracts in `packages/sdk` or a server API. No `modules/mod-A` importing `modules/mod-B` directly.
3. Shells (`apps/*`) do composition and platform concerns only — no domain logic.
4. Every module has an owner listed in `docs/modules/README.md`.

## Open decisions

Tracked in `docs/adr/`. At least: language/runtime per shell, transport (REST vs gRPC), persistence, auth, build tooling.
