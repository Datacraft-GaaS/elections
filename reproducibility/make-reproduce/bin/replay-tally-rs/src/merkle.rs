// merkle.rs -- BLAKE3-based Merkle chain for eel event streams.
//
// Chain construction: each event's canonical JSON bytes are hashed with BLAKE3,
// then chained: new_root = BLAKE3(prev_root || event_hash).
// The initial "previous root" is BLAKE3(b"") (the BLAKE3 hash of empty bytes).

pub struct MerkleChain {
    current: [u8; 32],
}

impl MerkleChain {
    pub fn new() -> Self {
        // Seed: BLAKE3 of empty bytes.
        let seed = *blake3::hash(b"").as_bytes();
        Self { current: seed }
    }

    /// Ingest one event's canonical JSON bytes into the chain.
    pub fn ingest(&mut self, event_bytes: &[u8]) {
        let event_hash = *blake3::hash(event_bytes).as_bytes();
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&self.current);
        combined[32..].copy_from_slice(&event_hash);
        self.current = *blake3::hash(&combined).as_bytes();
    }

    /// Return the current root as a lowercase hex string.
    pub fn finalize(&self) -> String {
        hex_encode(&self.current)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_chain_is_blake3_of_empty() {
        let chain = MerkleChain::new();
        let expected = hex_encode(blake3::hash(b"").as_bytes());
        assert_eq!(chain.finalize(), expected);
    }

    #[test]
    fn single_event_deterministic() {
        let mut c1 = MerkleChain::new();
        let mut c2 = MerkleChain::new();
        c1.ingest(b"hello");
        c2.ingest(b"hello");
        assert_eq!(c1.finalize(), c2.finalize());
    }

    #[test]
    fn order_matters() {
        let mut c1 = MerkleChain::new();
        let mut c2 = MerkleChain::new();
        c1.ingest(b"A");
        c1.ingest(b"B");
        c2.ingest(b"B");
        c2.ingest(b"A");
        assert_ne!(c1.finalize(), c2.finalize());
    }
}
