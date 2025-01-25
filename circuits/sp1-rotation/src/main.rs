#![no_main]
use alloy_primitives::FixedBytes;
use alloy_sol_types::SolType;
use committee_iso::types::{CommitteeUpdateArgs, PublicKeyHashes};
use committee_iso::utils::{
    commit_to_keys_with_sign, decode_pubkeys_x, hash_keys, merkleize_keys, uint64_to_le_256,
    verify_merkle_proof,
};
use rotation_iso::types::{RotationCircuitInputs, WrappedOutput};
use step_iso::types::SyncStepCircuitInput;
use step_iso::verify_aggregate_signature;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let inputs: RotationCircuitInputs = borsh::from_slice(&sp1_zkvm::io::read_vec()).unwrap();
    let committee_inputs: CommitteeUpdateArgs = inputs.committee;
    let step_inputs: SyncStepCircuitInput = inputs.step;
    let key_hashs: PublicKeyHashes = hash_keys(committee_inputs.pubkeys_compressed.clone());
    let committee_root_ssz: Vec<u8> = merkleize_keys(key_hashs);
    let finalized_state_root: Vec<u8> = committee_inputs.finalized_header.state_root.to_vec();
    let (keys, signs) = decode_pubkeys_x(committee_inputs.pubkeys_compressed);
    let commitment = commit_to_keys_with_sign(&keys, &signs);
    verify_merkle_proof(
        committee_inputs.sync_committee_branch,
        committee_root_ssz,
        &finalized_state_root,
        110,
    );
    let finalized_header_root_committee: Vec<u8> = merkleize_keys(vec![
        uint64_to_le_256(
            committee_inputs
                .finalized_header
                .slot
                .parse::<u64>()
                .unwrap(),
        ),
        uint64_to_le_256(
            committee_inputs
                .finalized_header
                .proposer_index
                .parse::<u64>()
                .unwrap(),
        ),
        committee_inputs.finalized_header.parent_root.to_vec(),
        finalized_state_root.clone(),
        committee_inputs.finalized_header.body_root.to_vec(),
    ]);

    verify_merkle_proof(
        step_inputs.args.execution_payload_branch.to_vec(),
        step_inputs.args.execution_payload_root.clone(),
        &step_inputs.args.finalized_header.body_root.to_vec(),
        9,
    );
    let finalized_header_root_step: Vec<u8> = merkleize_keys(vec![
        uint64_to_le_256(
            step_inputs
                .args
                .finalized_header
                .slot
                .parse::<u64>()
                .unwrap(),
        ),
        uint64_to_le_256(
            step_inputs
                .args
                .finalized_header
                .proposer_index
                .parse::<u64>()
                .unwrap(),
        ),
        step_inputs.args.finalized_header.parent_root.to_vec(),
        step_inputs.args.finalized_header.state_root.to_vec(),
        step_inputs.args.finalized_header.body_root.to_vec(),
    ]);
    // updates must be for the same root
    assert_eq!(finalized_header_root_committee, finalized_header_root_step);
    verify_merkle_proof(
        step_inputs.args.finality_branch.clone(),
        finalized_header_root_step.clone(),
        &step_inputs.args.attested_header.state_root.to_vec(),
        105,
    );
    verify_aggregate_signature(step_inputs.args.clone(), step_inputs.commitment);
    let output = WrappedOutput::abi_encode(&WrappedOutput {
        slot: u32::from_str_radix(&step_inputs.args.finalized_header.slot, 10)
            .expect("Failed to parse slot as u32"),
        // this should be the current sync committee commitment stored under the contract
        commitment: FixedBytes::<32>::from_slice(&step_inputs.commitment),
        finalized_header_root: FixedBytes::<32>::from_slice(&finalized_header_root_committee),
        // this should be the next sync committee, the output of this update
        next_commitment: FixedBytes::<32>::from_slice(&commitment),
    });
    sp1_zkvm::io::commit_slice(&output);
}
