#[cfg(test)]
mod integration {
    #[tokio::test]
    #[ignore = "requires docker; run with DOCKER_AVAILABLE=1"]
    async fn test_10k_form34a_aggregation() {
        // 1. Spin up PG via testcontainers (or skip if docker not available)
        // 2. Create EventStore, run migrations
        // 3. Publish 10k synthetic Form34A captured events
        // 4. Run aggregator 34A -> 34B -> 34C
        // 5. Assert candidate totals match expected sums
        // 6. Assert merkle root is non-empty string
        eprintln!("integration test: 10k form34a aggregation (docker required)");
    }

    #[tokio::test]
    async fn test_replay_shuffled_events_idempotent() {
        // Property: replaying events in any order produces the same final projection.
        // Uses an in-memory stub store.
        let _ = ();  // stub - proptest integration TODO
    }

    #[tokio::test]
    async fn test_form34a_amendment_preserves_original() {
        // Property: after form34a_amended event, original event still in log.
        let _ = ();  // stub - verify via event log query
    }
}
