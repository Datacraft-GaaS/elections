fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn leaf_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

pub fn compute_merkle_root(leaves: &[&[u8]]) -> String {
    if leaves.is_empty() {
        return encode_hex(&[0u8; 32]);
    }
    let mut hashes: Vec<[u8; 32]> = leaves.iter().map(|l| leaf_hash(l)).collect();
    while hashes.len() > 1 {
        let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
        let mut i = 0;
        while i < hashes.len() {
            if i + 1 < hashes.len() {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&hashes[i]);
                hasher.update(&hashes[i + 1]);
                next.push(*hasher.finalize().as_bytes());
            } else {
                next.push(hashes[i]);
            }
            i += 2;
        }
        hashes = next;
    }
    encode_hex(&hashes[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    #[trace("FR-M10-028")]
    #[test]
    fn empty_input_returns_zero_hash() {
        assert_eq!(compute_merkle_root(&[]), "00".repeat(32));
    }

    #[trace("FR-M10-028")]
    #[test]
    fn single_leaf_returns_leaf_hash() {
        let leaf = b"hello";
        assert_eq!(compute_merkle_root(&[leaf]), encode_hex(&leaf_hash(leaf)));
    }

    #[trace("FR-M10-028")]
    #[test]
    fn two_leaves_combine_and_reorder_changes_root() {
        let a = b"a";
        let b = b"b";
        let root_ab = compute_merkle_root(&[a, b]);
        let root_ba = compute_merkle_root(&[b, a]);
        assert_ne!(root_ab, root_ba);
        // root_ab == blake3(leaf_hash(a) || leaf_hash(b)).
        let mut hasher = blake3::Hasher::new();
        hasher.update(&leaf_hash(a));
        hasher.update(&leaf_hash(b));
        assert_eq!(root_ab, encode_hex(hasher.finalize().as_bytes()));
    }
}
