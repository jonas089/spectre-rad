use crate::utils::compute_digest;
use itertools::Itertools;
use lazy_static::lazy_static;

use ssz_rs::prelude::*;
pub type Root = Node;
pub type Slot = u64;
pub type ValidatorIndex = usize;
pub type PublicKeys = Vec<Vec<u8>>;
pub type PublicKeyHashes = Vec<Vec<u8>>;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BeaconBlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

pub type PublicKeysUncompressed = Vec<Vec<u8>>;
pub type SignatureCompressed = Vec<u8>;
pub type Branch = Vec<Vec<u8>>;
pub type Leaf = Vec<u8>;
pub type Commitment = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: PublicKeys,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: Branch,
}

lazy_static! {
    pub static ref ZERO_HASHES: [[u8; 32]; 2] = {
        std::iter::successors(Some([0; 32]), |&prev| {
            Some(compute_digest(&[prev, prev].concat()).try_into().unwrap())
        })
        .take(2)
        .collect_vec()
        .try_into()
        .unwrap()
    };
}
