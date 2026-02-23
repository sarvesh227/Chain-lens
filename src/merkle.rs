use sha2::{Digest, Sha256};

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second);
    result
}

pub fn compute_merkle_root(mut hashes: Vec<[u8; 32]>) -> [u8; 32] {
    if hashes.is_empty() {
        return [0u8; 32];
    }

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            let last = hashes.last().unwrap().clone();
            hashes.push(last);
        }

        let mut next_level = Vec::new();

        for pair in hashes.chunks(2) {
            let mut concat = Vec::new();
            concat.extend(&pair[0]);
            concat.extend(&pair[1]);

            next_level.push(double_sha256(&concat));
        }

        hashes = next_level;
    }

    hashes[0]
}