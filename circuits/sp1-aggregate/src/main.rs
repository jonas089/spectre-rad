#![no_main]
use aggregate_iso::types::{RecursiveInputs, WrappedOutput};
use alloy_primitives::FixedBytes;
use alloy_sol_types::SolType;
use committee_iso::types::CommitteeCircuitOutput;
use committee_iso::utils::{Digest, Sha256};
use step_iso::types::SyncStepCircuitOutput;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let inputs: Vec<RecursiveInputs> = borsh::from_slice(&sp1_zkvm::io::read_vec()).unwrap();
    let committee_inputs: RecursiveInputs = inputs.first().unwrap().clone();
    let step_inputs: RecursiveInputs = inputs.get(1).unwrap().clone();

    // verify committee proof
    sp1_zkvm::lib::verify::verify_sp1_proof(
        &committee_inputs.vk.clone(),
        &(Sha256::digest(&committee_inputs.public_values.clone())).into(),
    );
    // verify step proof
    sp1_zkvm::lib::verify::verify_sp1_proof(
        &step_inputs.vk.clone(),
        &(Sha256::digest(&step_inputs.public_values.clone())).into(),
    );
    let committee_journal: CommitteeCircuitOutput =
        borsh::from_slice(&committee_inputs.public_values).unwrap();
    let step_journal: SyncStepCircuitOutput =
        borsh::from_slice(&step_inputs.public_values).unwrap();

    // their roots must equal, the sync step signature must be for this committee update!
    assert_eq!(
        committee_journal.finalized_header_root,
        step_journal.finalized_header_root
    );

    let step_vk_bytes = step_inputs
        .vk
        .iter()
        .flat_map(|&x| x.to_le_bytes())
        .collect::<Vec<u8>>();

    let step_vk = FixedBytes::<32>::from_slice(&step_vk_bytes);

    let committee_vk_bytes = committee_inputs
        .vk
        .iter()
        .flat_map(|&x| x.to_le_bytes())
        .collect::<Vec<u8>>();

    let committee_vk = FixedBytes::<32>::from_slice(&committee_vk_bytes);

    let output = WrappedOutput::abi_encode(&WrappedOutput {
        slot: step_journal.slot,
        // this should be the current sync committee commitment stored under the contract
        commitment: FixedBytes::<32>::from_slice(&step_journal.commitment),
        finalized_header_root: FixedBytes::<32>::from_slice(&step_journal.finalized_header_root),
        step_vk,
        committee_vk,
        // this should be the next sync committee, the output of this update
        next_commitment: FixedBytes::<32>::from_slice(&committee_journal.commitment),
    });
    sp1_zkvm::io::commit_slice(&output);
}
