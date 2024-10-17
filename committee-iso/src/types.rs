use ethereum_consensus_types::BeaconBlockHeader;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::utils::digest;

pub type PublicKeys = Vec<Vec<u8>>;
pub type PublicKeyHashes = Vec<Vec<u8>>;
pub type CommitteeBranch = Vec<Vec<u8>>;
pub type Leaf = Vec<u8>;
pub type Root = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: PublicKeys,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: CommitteeBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeCircuitInput {
    pub pubkeys: Vec<Vec<u8>>,
    pub branch: Vec<Vec<u8>>,
    pub state_root: Vec<u8>,
}

lazy_static! {
    pub static ref ZERO_HASHES: [[u8; 32]; 2] = {
        std::iter::successors(Some([0; 32]), |&prev| {
            Some(digest(&[prev, prev].concat()).try_into().unwrap())
        })
        .take(2)
        .collect_vec()
        .try_into()
        .unwrap()
    };
}
