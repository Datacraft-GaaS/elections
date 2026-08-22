//! View and block store for HotStuff replicas.
//!
//! In production this is sqlx-backed (PostgreSQL). For tests we provide an
//! in-memory store. The replay-safe design stores the highest committed view
//! so a restarted replica can recover without re-processing old messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::Block;
use crate::BftError;

/// Persisted replica state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub replica_id: u64,
    pub last_committed_view: u64,
    pub last_committed_height: u64,
}

/// In-memory store (used in tests; replace with sqlx backend in production).
pub struct InMemoryStore {
    pub blocks: HashMap<u64, Block>,
    pub state: PersistedState,
}

impl InMemoryStore {
    pub fn new(replica_id: u64) -> Self {
        Self {
            blocks: HashMap::new(),
            state: PersistedState {
                replica_id,
                last_committed_view: 0,
                last_committed_height: 0,
            },
        }
    }

    /// Persist a committed block (replay-safe: no-op if already stored).
    pub fn save_block(&mut self, block: Block) -> Result<(), BftError> {
        self.blocks.entry(block.height).or_insert(block.clone());
        if block.height > self.state.last_committed_height {
            self.state.last_committed_height = block.height;
        }
        Ok(())
    }

    /// Retrieve a block by height.
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(&height)
    }

    /// Update the last committed view (call after each Decide phase).
    pub fn advance_view(&mut self, view: u64) {
        if view > self.state.last_committed_view {
            self.state.last_committed_view = view;
        }
    }

    /// Recover state after restart: returns the last committed height.
    pub fn recover(&self) -> PersistedState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_block(height: u64) -> Block {
        Block::new(height, [0u8; 32], format!("payload_{height}").into_bytes())
    }

    #[test]
    fn save_and_retrieve_block() {
        let mut store = InMemoryStore::new(0);
        let b = test_block(1);
        store.save_block(b.clone()).unwrap();
        assert_eq!(store.get_block(1).unwrap().height, 1);
    }

    #[test]
    fn save_is_idempotent() {
        let mut store = InMemoryStore::new(0);
        let b = test_block(2);
        store.save_block(b.clone()).unwrap();
        store.save_block(b.clone()).unwrap();
        assert_eq!(store.blocks.len(), 1);
    }

    #[test]
    fn recover_returns_last_height() {
        let mut store = InMemoryStore::new(0);
        store.save_block(test_block(1)).unwrap();
        store.save_block(test_block(2)).unwrap();
        let state = store.recover();
        assert_eq!(state.last_committed_height, 2);
    }

    #[test]
    fn advance_view_monotone() {
        let mut store = InMemoryStore::new(0);
        store.advance_view(3);
        store.advance_view(1); // should not regress
        assert_eq!(store.state.last_committed_view, 3);
    }
}
