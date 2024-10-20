use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::{env, fs};

use crate::{
    types::{CommitteeBranch, Leaf, PublicKeyHashes, PublicKeys, Root, ZERO_HASHES},
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

pub fn verify_merkle_proof(branch: CommitteeBranch, leaf: Leaf, root: &Root, mut gindex: usize) {
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

pub fn digest(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}
