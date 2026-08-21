# Event Flow

All inter-service communication flows through Kafka topics following the CloudEvents v1 schema defined in `packages/event-schemas/`. The outbox pattern (eel-outbox-*) guarantees at-least-once delivery; consumers are idempotent.

## Topic Naming

`eel.<service>` - one topic namespace per service. Examples: `eel.rts`, `eel.bvr`, `eel.bvi`, `eel.ecc`.

## Election-Day Critical Path

```mermaid
sequenceDiagram
    participant Kit as Field Kit (biometric-edge)
    participant BVI as bvi-svc
    participant ABIS as abis-svc
    participant RTS as rts-svc
    participant Transp as transparency-svc
    participant Portal as portal-svc

    Kit->>BVI: eel.kit.session.opened
    Kit->>ABIS: identification.attempt (template)
    ABIS-->>BVI: eel.abis.match.completed
    BVI-->>Kit: identification result
    Kit->>RTS: form34.received (signed scan)
    RTS->>RTS: ICR validation + Merkle anchor
    RTS-->>Transp: eel.rts.form34.published
    RTS-->>Transp: eel.rts.merkle_root.anchored
    Transp-->>Portal: bundle + inclusion proof
    Portal-->>Public: published form visible
```

## Register and Enrolment Flow

```mermaid
flowchart LR
    VPRS[vprs-svc] -->|application.submitted| BVR[bvr-svc]
    BiometricEdge[biometric-edge-svc] -->|enrollment.completed| BVR
    BVR -->|template.enrolled| ABIS[abis-svc]
    ABIS -->|adjudication.resolved| BVR
    BVR -->|register.snapshot.published| BVI[bvi-svc]
    BVR -->|register.snapshot.published| Transp[transparency-svc]
```

## Governance and Audit Flow

```mermaid
flowchart LR
    RTS[rts-svc] -->|form34.published| Gov[governance-svc]
    RTS -->|declaration.issued| Gov
    Transp[transparency-svc] -->|bundle.published| Gov
    ECC[ecc-svc] -->|grievance.resolved| Gov
    AI[ai-platform-svc] -->|alert.anomaly| ECC
    Gov -->|evidence_bundle.submitted| eCourt([eCourt])
    Gov -->|rla.run.completed| Portal[portal-svc]
```

## Disaster Recovery Event Replication

MirrorMaker 2 replicates all `eel.*` topics from primary to mirror site with sub-second lag. Edge clusters consume from the nearest regional mirror. See [deployment-topology.md](deployment-topology.md).
