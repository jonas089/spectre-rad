use committee_iso::utils::{merkleize_keys, uint64_to_le_256, verify_merkle_proof};
use risc0_zkvm::guest::env;
use std::io::Read;
use step_iso::{
    types::{SyncStepArgs, SyncStepCircuitInput, SyncStepCircuitOutput},
    verify_aggregate_signature,
};

fn main() {
    let mut buffer: Vec<u8> = vec![];
    let _ = env::stdin().read_to_end(&mut buffer);
    let inputs: SyncStepCircuitInput = borsh::from_slice(&buffer).unwrap();
    let args: SyncStepArgs = inputs.args;
    verify_merkle_proof(
        args.execution_payload_branch.to_vec(),
        args.execution_payload_root.clone(),
        &args.finalized_header.body_root.to_vec(),
        9,
    );

    let finalized_header_root = merkleize_keys(vec![
        uint64_to_le_256(args.finalized_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(args.finalized_header.proposer_index.parse::<u64>().unwrap()),
        args.finalized_header.parent_root.to_vec(),
        args.finalized_header.state_root.to_vec(),
        args.finalized_header.body_root.to_vec(),
    ]);

    verify_merkle_proof(
        args.finality_branch.clone(),
        finalized_header_root.clone(),
        &args.attested_header.state_root.to_vec(),
        105,
    );

    verify_aggregate_signature(args.clone(), inputs.committee_commitment);
    let output = SyncStepCircuitOutput {
        slot: u32::from_str_radix(&args.finalized_header.slot, 10).unwrap(),
        commitment: inputs.committee_commitment.try_into().unwrap(),
        finalized_header_root: finalized_header_root.try_into().unwrap(),
    };
    env::commit(&output);
}
