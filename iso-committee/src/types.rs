use crate::utils::digest;
use itertools::Itertools;
use lazy_static::lazy_static;

use ssz_rs::prelude::*;
pub type Root = Node;
pub type Slot = u64;
pub type ValidatorIndex = usize;
pub type PublicKeys = Vec<Vec<u8>>;
pub type PublicKeyHashes = Vec<Vec<u8>>;

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize,
)]
pub struct BeaconBlockHeader {
    #[serde(with = "ssz_rs::serde::as_str")]
    pub slot: Slot,
    #[serde(with = "ssz_rs::serde::as_str")]
    pub proposer_index: ValidatorIndex,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

pub type PublicKeysUncompressed = Vec<Vec<u8>>;
pub type SignatureCompressed = Vec<u8>;
pub type Branch = Vec<Vec<u8>>;
pub type Leaf = Vec<u8>;
pub type Commitment = Vec<u8>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: PublicKeys,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: Branch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitteeCircuitOutput {
    pub committee_root: Vec<u8>,
    pub commitment_pkeys: Vec<u8>,
    pub commitment_state_root: Vec<u8>,
}

impl CommitteeCircuitOutput {
    pub fn new(
        committee_root: Vec<u8>,
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
