# ADRs continued from 0025

This directory continues ADR numbering from 0025. It holds the newer hardware and
client-facing decisions. The earlier, indexed set lives in [`docs/adr/`](../../adr/),
which has its own README table.

## Index

| # | Subject | Status |
|---|---|---|
| [0025](0025-oss-rfid-stack.md) | UHF RFID readers: sllurp (LLRP) and bleak (BLE) behind a shared adapter | accepted |
| [0026](0026-hardware-latitude.md) | Hardware latitude via OSS driver stacks and stable driver abstractions | accepted |
| [0027](0027-desktop-bvr-client.md) | Desktop/laptop BVR registration client (Qt 6) and declarative enrolment workflows | accepted |

## Known numbering collision

**`0025` is defined twice with different subjects**: this directory's
`0025-oss-rfid-stack.md` (the RFID driver decision, which 0026 builds on) and
[`docs/adr/0025-factory-distribution-hosting.md`](../../adr/0025-factory-distribution-hosting.md).

Neither is wrong; the two directories were numbered independently. ADRs are
immutable after acceptance, so this is **not** fixed by renumbering, which would
rewrite accepted records and invalidate every citation. The collision is
recorded here and in [`docs/adr/README.md`](../../adr/README.md) instead.

Resolving it properly is a separate cleanup: either fold this directory's
contents into `docs/adr/` under fresh numbers with a superseding note, or declare
one directory canonical and migrate the other. That is a documentation decision,
not a code change, and should not be done piecemeal.

## Adding a new ADR

Append to the directory and add a row to the table above. Follow the house shape
used by 0025-0027: Context, Decision, Consequences, Alternatives, References.
Use the neutral `eel` identifier prefix in any code or package names.
