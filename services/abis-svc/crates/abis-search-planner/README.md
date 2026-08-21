# eel-abis-search-planner

Cost-based 1:N query planner for the eel ABIS across shards and accelerators.

## Algorithm

The planner evaluates all available search steps for a query:

1. **DemographicPrune**: filter gallery by age/sex/county (~5ms/M, keeps ~10%).
2. **FaceANN**: approximate nearest-neighbour in face embedding space (~50ms/M).
3. **IrisExact**: exact iris matcher on pruned candidates (~150ms/M).
4. **FingerprintExact**: exact minutiae matcher on pruned candidates (~200ms/M).

Steps are sorted cheapest-first (greedy cost minimisation). Only steps available
on a given shard are included.

## Early Termination

Execution stops when any shard returns a score >= `early_termination_threshold`,
indicating a confident match. This reduces latency from O(all shards) to O(first hit).

## Integration Points

- **ShardSpec**: describes which steps each shard supports.
- **CostModel**: tunable cost estimates per step per million gallery items.
- Consumer: `abis-matcher` calls the planner before dispatching to accelerators.

## Shard Parallelism

The executor runs shards concurrently (tokio::join in production). Early-termination
is cooperative: the first shard to breach the threshold stops further shard queries.

## Usage

```rust
use eel_abis_search_planner::plan::QueryPlanner;

let planner = QueryPlanner::default();
let plan = planner.plan(shards, 0.95)?;
```
