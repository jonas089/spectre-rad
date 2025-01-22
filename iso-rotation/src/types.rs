use alloy_sol_types::sol;
use borsh::{BorshDeserialize, BorshSerialize};
use committee_iso::types::CommitteeUpdateArgs;
use serde::{Deserialize, Serialize};
use step_iso::types::SyncStepCircuitInput;

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RotationCircuitInputs {
    pub committee: CommitteeUpdateArgs,
    pub step: SyncStepCircuitInput,
}

sol! {
    struct WrappedOutput{
        uint32 slot;
        bytes32 commitment;
        bytes32 finalized_header_root;
        bytes32 next_commitment;
    }
}
