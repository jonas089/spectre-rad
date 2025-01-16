pub type Node = ssz_rs::Node;
pub type ValidatorIndex = usize;
use alloy_sol_types::sol;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct BeaconBlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: Node,
    pub state_root: Node,
    pub body_root: Node,
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
    pub slot: u32,
    pub commitment: [u8; 32],
    pub finalized_header_root: [u8; 32],
}

sol! {
    struct WrappedOutput{
        uint32 slot;
        bytes32 commitment;
        bytes32 finalized_header_root;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RecursiveInputs {
    pub public_values: Vec<u8>,
    pub vk: [u32; 8],
}
