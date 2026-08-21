# eel-governance-review

M41 Post-election review: findings documentation, recommendations tracking, parliament referrals, long-drip alert escalation.

## What it does

Manages post-election review findings and recommendations: IEBC internal review teams document issues found during election operations, tag recommendations (procedural changes, policy, training, technology), escalate significant findings (>3 counties affected or T1 system failure) to Parliament, and track implementation status. Integrates with incident reporting (field-mobile.incident.created events) and RLA audit findings (governance.rla.recount.triggered events).

Fits in governance-svc as the findings aggregation layer. Runs on port 8094 (shared with reporting); emits governance.parliament.referral.issued on escalation; consumes field-mobile and governance events.

## Public API

`eel_governance_review::finding::Finding` aggregate: `fn log_finding(&mut self, issue: Issue, tags: Vec<Tag>) -> Result<()>`, `fn add_recommendation(&mut self, rec: Recommendation) -> Result<()>`, `fn escalate_to_parliament(&mut self) -> Result<()>`.

Endpoints:
- `GET /v1/governance/review/findings` - list findings (filter by status, tag)
- `POST /v1/governance/review/findings` - log new finding
- `GET /v1/governance/review/findings/{id}` - fetch finding + recommendations
- `POST /v1/governance/review/findings/{id}/escalate` - escalate to parliament
- `GET /v1/governance/review/implementation-status` - track recommendation progress

Escalations automatically attach to StatutoryReport (M58) for parliament.

## Feature flags

`#[cfg(feature = "elasticsearch")]` for full-text search of findings; tests use in-memory substring search.

## Test commands

```bash
cargo test -p eel-governance-review --all-features
cargo test -p eel-governance-review --lib escalation_rules
cargo test -p eel-governance-review --test integration_review -- --include-ignored
```

Test escalation thresholds (3+ counties or T1 severity); verify event consumption and recommendation tagging.

## Bench commands

```bash
cargo bench -p eel-governance-review search_10k_findings
```

Full-text search of 10k findings should complete in < 100ms.

## Dependencies of note

- `uuid6`: finding and recommendation IDs
- `opensearch`: full-text search index
- `chrono`: creation and update timestamps
- `serde_json`: finding and tag parsing

## Related docs

- `docs/api-reference.md#review` (escalation rule diagram)
- `docs/architecture.md#post-mortem` (review workflow, timeline)
- `migration_009_review.sql` (schema: findings, recommendations, parliament_referrals, implementation_status)
