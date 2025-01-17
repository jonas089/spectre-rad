use alloy_sol_types::sol;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

sol! {
    struct WrappedOutput{
        uint32 slot;
        bytes32 commitment;
        bytes32 finalized_header_root;
        bytes32 step_vk;
        bytes32 committee_vk;
        bytes32 next_commitment;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RecursiveInputs {
    pub public_values: Vec<u8>,
    pub vk: [u32; 8],
}
