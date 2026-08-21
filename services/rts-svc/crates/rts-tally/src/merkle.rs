fn encode_hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

pub fn leaf_hash(data: &[u8]) -> [u8; 32] { *blake3::hash(data).as_bytes() }

pub fn compute_merkle_root(leaves: &[&[u8]]) -> String {
    if leaves.is_empty() { return encode_hex(&[0u8; 32]); }
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
