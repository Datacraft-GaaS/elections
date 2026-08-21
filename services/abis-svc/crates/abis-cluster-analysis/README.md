# eel-abis-cluster-analysis

Multi-way duplicate cluster detection for the eel ABIS using connected-components analysis.

## Algorithm

1. Build a match graph: nodes are subject IDs, edges are adjudication-confirmed
   match pairs from the last 30 days.
2. Run union-find connected-components to identify groups of mutually-linked subjects.
3. Any component with >= 2 members is a duplicate cluster requiring resolution.

Union-find (path compression + rank) runs in near-linear time O(n alpha(n)),
suitable for millions of edges.

## Resolution Paths

| Action | Meaning |
|--------|---------|
| merge_into_one | All members are the same person, consolidate records |
| split_into_groups | Separate into distinct identity groups |
| false_alarm | Cluster is explained by twins or other legitimate coincidence |

## Integration Points

- **eel_duplicate_clusters** table: one row per cluster.
- Nightly job: calls `detect_clusters` over the last 30 days of adjudication data.
- Consumer: operations dashboard cluster resolution workflow.

## SLA Impact

Unresolved clusters older than 7 days trigger escalation to Tier-2 adjudicators.
