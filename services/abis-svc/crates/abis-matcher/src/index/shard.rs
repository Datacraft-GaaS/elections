//! 256-shard hasher for distributing subjects across HNSW graph shards.
//!
//! `ShardId = blake3(subject_id)[0] mod 256` — maps each subject deterministically
//! to one of 256 independent index shards.

/// Shard identifier in [0, 255].
pub type ShardId = u8;

/// Compute the shard ID for a subject from its UUID string.
pub fn shard_id(subject_id: &str) -> ShardId {
    let hash = blake3::hash(subject_id.as_bytes());
    hash.as_bytes()[0]
}

/// Compute a distribution of shard IDs for a list of subject IDs.
///
/// Returns a 256-element array where `counts[i]` = number of subjects mapped to shard `i`.
pub fn shard_distribution(subject_ids: &[&str]) -> [u32; 256] {
    let mut counts = [0u32; 256];
    for id in subject_ids {
        counts[shard_id(id) as usize] += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_deterministic() {
        assert_eq!(shard_id("voter-001"), shard_id("voter-001"));
    }

    #[test]
    fn shard_range() {
        for i in 0..1000u32 {
            let id = format!("subject-{i}");
            let _s: u8 = shard_id(&id);
        }
    }
}
