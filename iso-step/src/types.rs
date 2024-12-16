pub type Root = ssz_rs::Node;
pub type Slot = u64;
pub type ValidatorIndex = usize;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
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
pub type Commitment = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SyncStepArgs {
    pub signature_compressed: SignatureCompressed,
    pub pubkeys_uncompressed: PublicKeysUncompressed,
    pub pariticipation_bits: Vec<bool>,
    pub attested_header: BeaconBlockHeader,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Branch,
    pub execution_payload_root: Vec<u8>,
    pub execution_payload_branch: Branch,
    pub domain: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SyncStepCircuitInput {
    pub args: SyncStepArgs,
    pub committee_commitment: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SyncStepCircuitOutput {
    pub finalized_block_root: [u8; 32],
}
