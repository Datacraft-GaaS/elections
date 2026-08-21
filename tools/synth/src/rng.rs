// Seeded deterministic RNG helpers.
// ChaCha20 is the canonical PRNG for this project. Same seed, same stream.

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Create a new ChaCha20 RNG seeded from a u64.
/// To create independent streams for different generators, derive sub-seeds
/// by calling `child(tag)` rather than sharing the parent RNG.
pub fn seeded(seed: u64) -> ChaCha20Rng {
    // Expand u64 seed to 32-byte key deterministically.
    let mut key = [0u8; 32];
    let bytes = seed.to_le_bytes();
    for (i, b) in bytes.iter().enumerate() {
        key[i] = *b;
        key[i + 8] = b.wrapping_add(0x5a);
        key[i + 16] = b.wrapping_add(0xa5);
        key[i + 24] = b.wrapping_add(0xf0);
    }
    ChaCha20Rng::from_seed(key)
}

/// Derive a child seed for a named sub-generator.
/// Uses a simple mixing function: hash(tag) XOR parent seed.
pub fn child_seed(parent: u64, tag: &str) -> u64 {
    let h: u64 = tag.bytes().fold(0xcbf29ce484222325_u64, |acc, b| {
        acc.wrapping_mul(0x100000001b3).wrapping_add(b as u64)
    });
    parent.wrapping_add(h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn same_seed_identical_stream() {
        let mut a = seeded(42);
        let mut b = seeded(42);
        let mut buf_a = [0u8; 64];
        let mut buf_b = [0u8; 64];
        a.fill_bytes(&mut buf_a);
        b.fill_bytes(&mut buf_b);
        assert_eq!(buf_a, buf_b);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = seeded(1);
        let mut b = seeded(2);
        let mut buf_a = [0u8; 8];
        let mut buf_b = [0u8; 8];
        a.fill_bytes(&mut buf_a);
        b.fill_bytes(&mut buf_b);
        assert_ne!(buf_a, buf_b);
    }

    #[test]
    fn child_seed_is_deterministic() {
        assert_eq!(child_seed(42, "voters"), child_seed(42, "voters"));
        assert_ne!(child_seed(42, "voters"), child_seed(42, "geo"));
    }
}
