# eel-abis-twin-registry

Twin group management and automated detection for the eel ABIS.

## Problem

Identical twins score in an ambiguous zone during 1:N matching: high enough to
surface as candidates but below the match threshold. Without a twin registry,
operators cannot distinguish genuine twins from borderline impostors.

## Twin Candidate Detection

During 1:N search, pairs scoring between 0.60 and 0.85 with demographic or
geographic overlap are flagged as TwinCandidate records. Operators review and
create TwinGroup records from confirmed candidates.

## Face-Iris Correlation Check

Genuine twins share high face similarity but show moderate iris divergence (iris
patterns are not genetically identical). The correlation check accepts pairs with
high face score AND non-trivial iris delta (> 0.05).

## Civil Registry Sync

TwinGroup records are cross-referenced with Kenya civil registry via API stub.
Once all members have civil_registry_id populated, status transitions to
VerifiedByCivilRegistry.

## Integration Points

- `eel_twin_registry` table: group records.
- `eel_twin_members` table: per-subject membership rows.
- Consumer: 1:N matcher, operations dashboard.

## Usage

```rust
use eel_abis_twin_registry::registry::create_twin_group;

let group = create_twin_group(&[twin_a_id, twin_b_id])?;
```
