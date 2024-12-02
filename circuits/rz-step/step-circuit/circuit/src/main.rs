use committee_iso::{
    types::PublicKeyHashes,
    utils::{decode_pubkeys_x, hash_keys, merkleize_keys, uint64_to_le_256, verify_merkle_proof},
};
use risc0_zkvm::guest::env;
use step_iso::types::{SyncStepArgs, SyncStepCircuitOutput};
use step_iso::verify_aggregate_signature;

fn main() {
    let args: SyncStepArgs = env::read();
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

    let aggregate_key_commitment: Vec<u8> = verify_aggregate_signature(args.clone());
    let output = SyncStepCircuitOutput {
        finalized_block_root: finalized_header_root,
        aggregate_key_commitment,
    };
    env::commit(&output);
}
