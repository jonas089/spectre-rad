#![no_main]

use committee_iso::utils::{merkleize_keys, uint64_to_le_256, verify_merkle_proof};
use step_iso::{types::SyncStepArgs, verify_aggregate_signature};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let args: SyncStepArgs = serde_json::from_slice(&sp1_zkvm::io::read_vec()).unwrap();
    verify_merkle_proof(
        args.execution_payload_branch.to_vec(),
        args.execution_payload_root.clone(),
        &args.finalized_header.body_root.to_vec(),
        9,
    );

    let finalized_header_root = merkleize_keys(vec![
        uint64_to_le_256(args.finalized_header.slot),
        uint64_to_le_256(args.finalized_header.proposer_index as u64),
        args.finalized_header.parent_root.to_vec(),
        args.finalized_header.state_root.to_vec(),
        args.finalized_header.body_root.to_vec(),
    ]);

    verify_merkle_proof(
        args.finality_branch.clone(),
        finalized_header_root,
        &args.attested_header.state_root.to_vec(),
        105,
    );

    verify_aggregate_signature(args.clone());
}
