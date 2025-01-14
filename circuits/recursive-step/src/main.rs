#![no_main]
use committee_iso::utils::{merkleize_keys, uint64_to_le_256, verify_merkle_proof};
use step_iso::{
    types::{SyncStepArgs, SyncStepCircuitInput, SyncStepCircuitOutput},
    verify_aggregate_signature,
};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    // read public values (and commit for verification)
    // read verifying key (and commit for verification)
    // verify proof using syscall
}
