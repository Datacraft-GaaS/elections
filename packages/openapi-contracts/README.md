# openapi-contracts

OpenAPI 3.1 specifications for every eel service API surface, versioned per bounded context. Drives all client generation.

## Purpose in context

All APIs in the platform are spec-first (ADR-0011). Specifications live here; services implement against them. A Spectral ruleset enforces ADR-0011 constraints: tier annotations, RFC 9457 problem schemas, JSON+CBOR content negotiation, signing headers, and mandatory `Idempotency-Key` on mutating operations. Clients in all four languages are regenerated from these specs by `just gen-clients`.

## Layout

```
openapi-contracts/
  _shared/
    common.yaml           # shared components: pagination, errors, HLC, UUIDv7
    components.yaml       # reusable schemas: Envelope, DataClass, Subject
    headers.yaml          # standard request/response headers
    problem-types.yaml    # RFC 9457 type URIs for all error classes
    problem-types-t2.yaml # Tier-2 problem types
  rts/
    rts.yaml              # Results Transmission System API
  bvr/
    bvr.yaml              # Biometric Voter Registration API
  portal/
    portal.yaml           # Voter/candidate portal API
  transparency/
    transparency.yaml     # Transparency log API
  # ... one directory per bounded context
  .spectral.yaml          # Spectral ruleset enforcing ADR-0011
```

## Contract conventions (ADR-0011)

- Every path tagged with `x-eel-tier: A|B|C` matching data-class
- Mutating operations include `Idempotency-Key` header reference from `_shared/headers.yaml`
- Responses include `X-Eel-Signature` header for Tier-A operations
- Error responses reference `problem-types.yaml` `type` URIs only
- Content negotiation: `application/json` and `application/cbor` on all endpoints

## Regenerating clients

```bash
# Regenerate all clients from specs:
just gen-clients

# Regenerate a single bounded context:
just gen-clients --context rts

# Lint all specs against the Spectral ruleset:
just lint-contracts
```

## Verify

```bash
# Check all specs parse and pass Spectral:
npx @stoplight/spectral-cli lint openapi-contracts/**/*.yaml --ruleset openapi-contracts/.spectral.yaml

# Check generated clients are up to date:
just check-clients-fresh
```

## Adding a new bounded context

1. Create `openapi-contracts/<ctx>/<ctx>.yaml`
2. Reference shared components from `_shared/`
3. Run `just lint-contracts` and fix all Spectral errors
4. Run `just gen-clients` to regenerate `packages/api-clients/`
5. Commit both the spec and the generated clients in the same PR

## Related docs

- `docs/adr/0011-api-conventions.md` - full API conventions and tier rules
- `packages/api-clients/README.md` - generated client packages (do not hand-edit)
- `packages/event-schemas/README.md` - CloudEvent schemas (separate from REST APIs)

## Limitations

Generated clients in `packages/api-clients/` are committed to the repo for reproducibility. They must not be hand-edited; changes to the generated files will be overwritten by `just gen-clients`. The Spectral ruleset enforces structural constraints but cannot verify semantic correctness; that is the responsibility of contract tests in each service.
