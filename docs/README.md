# docs

- `workforce-lifecycle.md`, poll-worker lifecycle design: recruitment, documents, competency exams, registry, postings, tasking, equipment, performance
- `abis/`, **ABIS biometric platform reference**: 85-crate matcher/algorithm inventory across fingerprint, iris, face, security & standards volumes
- `system/`, **complete implementation-referenced system reference**: architecture, security & safeguards, operations, shared packages, tools, and one page per service (start at `system/README.md`)
- `architecture/`, earlier system-wide design prose (overview, boundaries, data flow)
- `adr/`, architecture decision records, one file per decision, numbered
- `research/`, spikes and external-integration research
- `plans/`, the full build plan (waves, work packages, edge-case matrix, DoD, completeness audit, per-module pages)
- `spec-alignment/`, edits to apply to the bid documents so they match the ADRs
- `api/`, HTTP/gRPC/protobuf/OpenAPI contracts
- `modules/`, index of the 69 modules, home services, owners
- `guides/`, how-to for contributors (setup, workflow, release)

Package-level references live with the code they describe. Notably
`packages/eel-bvr-workflow-rs/docs/`, which holds the **BVR enrolment workflow
DSL reference** and its **training course**: the declarative language that
generates the voter registration form, so a new data item is a document edit
rather than a client release. See
[ADR-0027](architecture/decisions/0027-desktop-bvr-client.md).
- `runbooks/`, on-call / operational procedures
