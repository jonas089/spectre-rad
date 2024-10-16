use std::fs;

use ethereum_consensus_types::BeaconBlockHeader;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: Vec<Vec<u8>>,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: Vec<Vec<u8>>,
}

fn load_circuit_args() -> CommitteeUpdateArgs {
    serde_json::from_slice(&fs::read("./data/rotation_512.json").unwrap()).unwrap()
}

fn main() {
    let args = load_circuit_args();
    println!("Circuit args: {:?}", &args.finalized_header);
    let key_hashs = hash_keys(args.pubkeys_compressed.clone());
    let committee_root_ssz = merkleize_keys(key_hashs);
    let finalized_state_root = args.finalized_header.state_root.to_vec();

    verify_merkle_proof(
        args.sync_committee_branch,
        committee_root_ssz,
        &finalized_state_root,
        110,
    );

    println!("Yippie!");
}

fn verify_merkle_proof(branch: Vec<Vec<u8>>, leaf: Vec<u8>, root: &[u8], mut gindex: usize) {
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

fn hash_keys(keys: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut key_hashes: Vec<Vec<u8>> = vec![];
    for key in keys {
        let mut padded_key = key.clone();
        padded_key.resize(64, 0);
        key_hashes.push(digest(&padded_key));
    }
    key_hashes
}

fn digest(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

fn merkleize_keys(mut keys: Vec<Vec<u8>>) -> Vec<u8> {
    println!("Keys: {:?}", &keys.len());
    let height = if keys.len() == 1 {
        1
    } else {
        keys.len().next_power_of_two().ilog2() as usize
    };
    println!("Height: {}", &height);

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
    // returns root of a tree constructed from keys
    keys.pop().unwrap()
}

fn add_left_right(left: Vec<u8>, right: &[u8]) -> Vec<u8> {
    let mut value: Vec<u8> = left;
    value.extend_from_slice(&right);
    value.to_vec()
}

pub fn is_valid_merkle_branch(
    leaf: Vec<u8>,
    branch: Vec<Vec<u8>>,
    depth: u64,
    index: u64,
    root: Vec<u8>,
) -> bool {
    if branch.len() != depth as usize {
        return false;
    }
    let mut value = leaf.clone();
    if leaf.len() < 32 as usize {
        return false;
    }
    for i in 0..depth {
        if branch[i as usize].len() < 32 as usize {
            return false;
        }
        if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
            let mut data = [0u8; 64];
            data[0..32].copy_from_slice(&(value));
            data[32..64].copy_from_slice(&(branch[i as usize]));
            value = digest(&data).into();
        } else {
            let mut data = [0u8; 64]; // right node
            data[0..32].copy_from_slice(&(branch[i as usize]));
            data[32..64].copy_from_slice(&(value));
            value = digest(&data).into();
        }
    }
    value == root
}

lazy_static! {
    static ref ZERO_HASHES: [[u8; 32]; 2] = {
        std::iter::successors(Some([0; 32]), |&prev| {
            Some(
                sha2::Sha256::digest([prev, prev].concat())
                    .to_vec()
                    .try_into()
                    .unwrap(),
            )
        })
        .take(2)
        .collect_vec()
        .try_into()
        .unwrap()
    };
}
