use crate::utils::digest;
use ethereum_consensus_types::BeaconBlockHeader;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub type PublicKeys = Vec<Vec<u8>>;
pub type PublicKeyHashes = Vec<Vec<u8>>;
pub type Branch = Vec<Vec<u8>>;
pub type Leaf = Vec<u8>;
pub type Root = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: PublicKeys,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: Branch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeCircuitOutput {
    pub committee_root: Root,
    pub commitment_pkeys: Vec<u8>,
    pub commitment_state_root: Vec<u8>,
}

impl CommitteeCircuitOutput {
    pub fn new(
        committee_root: Root,
        commitment_pkeys: [u8; 32],
        commitment_state_root: Vec<u8>,
    ) -> Self {
        Self {
            committee_root,
            commitment_pkeys: commitment_pkeys.to_vec(),
            commitment_state_root,
        }
    }
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
