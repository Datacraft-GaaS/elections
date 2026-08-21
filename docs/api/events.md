# Event Schemas

All event schemas follow CloudEvents v1 and live under `packages/event-schemas/`. Schema files use JSON Schema draft-07.

## Schema Index

| Schema namespace | File path | Producer service | Key consumers |
|---|---|---|---|
| abis | `packages/event-schemas/abis/` | abis-svc | bvr-svc, biometric-edge-svc |
| ai_platform | `packages/event-schemas/ai_platform/` | ai-platform-svc | ecc-svc, governance-svc |
| ballot_serial | `packages/event-schemas/ballot_serial/` | rts-svc | inventory-svc, governance-svc |
| bvi | `packages/event-schemas/bvi/` | bvi-svc | rts-svc, ecc-svc, queue-svc, portal-svc |
| bvr | `packages/event-schemas/bvr/` | bvr-svc | bvi-svc, abis-svc, vprs-svc, transparency-svc, portal-svc |
| credential | `packages/event-schemas/credential/` | bvr-svc | field-mobile-svc |
| crms | `packages/event-schemas/crms/` | crms-svc | inventory-svc, portal-svc, transparency-svc |
| custody | `packages/event-schemas/custody/` | inventory-svc | ecc-svc, governance-svc |
| declaration | `packages/event-schemas/declaration/` | rts-svc | governance-svc, portal-svc |
| field_mobile | `packages/event-schemas/field_mobile/` | field-mobile-svc | ecc-svc, support-admin-svc |
| governance | `packages/event-schemas/governance/` | governance-svc | portal-svc, training-svc |
| grievance | `packages/event-schemas/grievance/` | ecc-svc | governance-svc, transparency-svc |
| inventory | `packages/event-schemas/inventory/` | inventory-svc | biometric-edge-svc, bvi-svc, rts-svc, ecc-svc |
| kit | `packages/event-schemas/kit/` | biometric-edge-svc | bvr-svc, abis-svc, bvi-svc, rts-svc |
| mobile | `packages/event-schemas/mobile/` | field-mobile-svc | ecc-svc, support-admin-svc |
| party | `packages/event-schemas/party/` | crms-svc | transparency-svc, portal-svc |
| portal | `packages/event-schemas/portal/` | portal-svc | ecc-svc |
| queue | `packages/event-schemas/queue/` | queue-svc | ecc-svc |
| rts | `packages/event-schemas/rts/` | rts-svc | transparency-svc, portal-svc, governance-svc, ecc-svc |
| support_admin | `packages/event-schemas/support_admin/` | support-admin-svc | governance-svc |
| training | `packages/event-schemas/training/` | training-svc | field-mobile-svc, portal-svc |
| transparency | `packages/event-schemas/transparency/` | transparency-svc | portal-svc, governance-svc |
| vprs | `packages/event-schemas/vprs/` | vprs-svc | bvr-svc, biometric-edge-svc |

## CloudEvents Envelope

Every event published to Kafka wraps the domain payload in a CloudEvents envelope:

```json
{
  "specversion": "1.0",
  "type": "eel.<service>.<entity>.<verb>",
  "source": "//eel/<service>",
  "id": "<uuid7>",
  "time": "<rfc3339>",
  "datacontenttype": "application/json",
  "data": { ... }
}
```

Type naming convention: `eel.<service>.<entity>.<verb>`. Examples:
- `eel.rts.form34.published`
- `eel.bvr.voter.registered`
- `eel.abis.match.completed`

## Schema Registry

The schema registry at `packages/event-schemas/` is enforced by CI: any breaking schema change (field removal, type narrowing) fails the schema-compatibility gate. Adding optional fields is non-breaking.

## Related

- [API Contracts](README.md)
- [Event flow diagram](../architecture/cross-cutting/event-flow.md)
- `docs/EVENT-CONVENTIONS.md` - detailed naming and evolution conventions
