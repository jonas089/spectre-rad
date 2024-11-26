use itertools::Itertools;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use std::{env, fs};

use crate::{
    types::{Branch, Leaf, PublicKeyHashes, PublicKeys, ZERO_HASHES},
    CommitteeUpdateArgs,
};

pub fn load_circuit_args(path: &str) -> CommitteeUpdateArgs {
    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap()
}

pub fn load_circuit_args_env() -> CommitteeUpdateArgs {
    let path =
        env::var("COMMITTEE_UPDATE_TEST_PATH").unwrap_or("../data/rotation_512.json".to_string());
    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap()
}

pub fn verify_merkle_proof(branch: Branch, leaf: Leaf, root: &Vec<u8>, mut gindex: usize) {
    let mut computed_hash = leaf;
    for node in branch {
        if gindex % 2 == 0 {
            computed_hash = digest(&add_left_right(computed_hash, &node));
        } else {
            computed_hash = digest(&add_left_right(node, &computed_hash));
        }
        gindex /= 2;
    }
    assert_eq!(&computed_hash, root);
}

// for the step circuit the PublicKeyHashes are generic Hashes.
// todo: make this more intuitive.
pub fn merkleize_keys(mut keys: PublicKeyHashes) -> Vec<u8> {
    let height = if keys.len() == 1 {
        1
    } else {
        keys.len().next_power_of_two().ilog2() as usize
    };

    for depth in 0..height {
        let len_even: usize = keys.len() + keys.len() % 2;
        let padded_keys = keys
            .into_iter()
            .pad_using(len_even, |_| ZERO_HASHES[depth].as_slice().to_vec())
            .collect_vec();
        keys = padded_keys
            .into_iter()
            .tuples()
            .map(|(left, right)| digest(&add_left_right(left, &right)))
            .collect::<Vec<Vec<u8>>>();
    }
    keys.pop().unwrap()
}

pub fn add_left_right(left: Leaf, right: &Leaf) -> Vec<u8> {
    let mut value: Vec<u8> = left;
    value.extend_from_slice(&right);
    value.to_vec()
}

pub fn hash_keys(keys: PublicKeys) -> PublicKeyHashes {
    let mut key_hashes: PublicKeyHashes = vec![];
    for key in keys {
        let mut padded_key = key.clone();
        padded_key.resize(64, 0);
        key_hashes.push(digest(&padded_key));
    }
    key_hashes
}

pub fn decode_pubkeys_x(
    compressed_encodings: impl IntoIterator<Item = Vec<u8>>,
) -> (Vec<BigUint>, Vec<u8>) {
    let (x_coordinates, y_signs): (Vec<_>, Vec<_>) = compressed_encodings
        .into_iter()
        .map(|mut bytes| {
            assert_eq!(bytes.len(), 48);
            let (x_coordinate_bytes, remaining) = bytes.split_at_mut(32);
            let masked_byte = remaining[15];
            let cleared_byte = masked_byte & 0x1F;
            let y_sign = (masked_byte >> 5) & 1;
            remaining[15] = cleared_byte;
            let x_coordinate = BigUint::from_bytes_be(x_coordinate_bytes);
            (x_coordinate, y_sign)
        })
        .unzip();

    let y_signs_packed = y_signs
        .chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0, |acc, (i, &bit)| acc | (bit << i))
        })
        .collect();

    (x_coordinates, y_signs_packed)
}

pub fn digest(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn commit_to_keys(keys: Vec<BigUint>, signs: Vec<u8>) -> [u8; 32] {
    let mut input: Vec<u8> = vec![];
    for key in keys {
        input.extend_from_slice(&key.to_bytes_be());
    }
    input.extend_from_slice(&signs);
    digest(&input).try_into().unwrap()
}

pub fn uint64_to_le_256(value: u64) -> Vec<u8> {
    let mut bytes = value.to_le_bytes().to_vec(); // Convert to little-endian 8 bytes
    bytes.extend(vec![0u8; 24]); // Pad with 24 zeros to make it 32 bytes
    bytes
}

#[cfg(test)]
mod tests {
    use super::{decode_pubkeys_x, load_circuit_args_env};
    use crate::{types::CommitteeUpdateArgs, utils::commit_to_keys};

    #[test]
    fn test_poseidon_commit() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let compressed: (Vec<num_bigint::BigUint>, Vec<u8>) =
            decode_pubkeys_x(args.pubkeys_compressed.clone());
        let commitment = commit_to_keys(compressed.0, compressed.1);
        println!("Commitment: {:?}", &commitment);
    }

    #[test]
    fn test_precompile_commit() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let compressed: (Vec<num_bigint::BigUint>, Vec<u8>) =
            decode_pubkeys_x(args.pubkeys_compressed.clone());
        let commitment = commit_to_keys(compressed.0, compressed.1);
        println!("Commitment: {:?}", &commitment);
    }
}
