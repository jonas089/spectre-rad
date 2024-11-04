use ark_bn254::Fr;
use itertools::Itertools;
use light_poseidon::Poseidon;
use light_poseidon::PoseidonBytesHasher;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::{env, fs};

use crate::constants::DEFAULT_FIELD_MODULUS;
use crate::{
    types::{Branch, Leaf, PublicKeyHashes, PublicKeys, Root, ZERO_HASHES},
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

pub fn verify_merkle_proof(branch: Branch, leaf: Leaf, root: &Root, mut gindex: usize) {
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
pub fn merkleize_keys(mut keys: PublicKeyHashes) -> Root {
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

pub fn precompile_sha2_commitment(keys: Vec<BigUint>, signs: Vec<u8>) -> [u8; 32] {
    let mut input: Vec<u8> = vec![];
    for key in keys {
        input.extend_from_slice(&key.to_bytes_be());
    }
    input.extend_from_slice(&signs);
    digest(&input).try_into().unwrap()
}

pub fn poseidon_commit_pubkeys_compressed(keys: Vec<BigUint>, signs: Vec<u8>) -> [u8; 32] {
    // maximum chunks with circom config: 13
    // todo: hash more chunks per round
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
    let mut input: Vec<Vec<u8>> = vec![
        (keys.get(0).unwrap() % BigUint::from_str(&DEFAULT_FIELD_MODULUS).unwrap()).to_bytes_be(),
        vec![],
    ];

    for chunk in signs {
        let bits = u8_to_bits(chunk);
        for bit in bits {
            let mut padded: Vec<u8> = vec![0; 31];
            padded.push(bit);
            input[1] = padded;
            input[0] = poseidon
                .hash_bytes_be(&input.iter().map(|array| &array[..]).collect::<Vec<&[u8]>>()[..])
                .unwrap()
                .to_vec();
        }
    }
    input.get(0).unwrap().as_slice().try_into().unwrap()
}

fn u8_to_bits(byte: u8) -> [u8; 8] {
    let mut bits = [0u8; 8];
    for i in 0..8 {
        bits[7 - i] = (byte >> i) & 1;
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::{decode_pubkeys_x, load_circuit_args_env, poseidon_commit_pubkeys_compressed};
    use crate::types::CommitteeUpdateArgs;
    use ark_bn254::Fr;
    use light_poseidon::{Poseidon, PoseidonBytesHasher};

    #[test]
    fn test_poseidon_commit() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let compressed: (Vec<num_bigint::BigUint>, Vec<u8>) =
            decode_pubkeys_x(args.pubkeys_compressed.clone());
        let commitment = poseidon_commit_pubkeys_compressed(compressed.0, compressed.1);
        println!("Commitment: {:?}", &commitment);
    }

    #[test]
    fn test_precompile_commit() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let compressed: (Vec<num_bigint::BigUint>, Vec<u8>) =
            decode_pubkeys_x(args.pubkeys_compressed.clone());
        let commitment = poseidon_commit_pubkeys_compressed(compressed.0, compressed.1);
        println!("Commitment: {:?}", &commitment);
    }
    #[test]
    fn poseidon_setup() {
        // BigUint is 32 Bytes
        let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
        let hash = poseidon.hash_bytes_be(&[&[1u8; 32], &[2u8; 32]]).unwrap();
        println!("{:?}", &hash);
    }
}
