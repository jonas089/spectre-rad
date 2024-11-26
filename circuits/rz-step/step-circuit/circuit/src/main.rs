use committee_iso::{
    types::{PublicKeyHashes, Root},
    utils::{
        decode_pubkeys_x, hash_keys, merkleize_keys, precompile_sha2_commitment, uint64_to_le_256,
        verify_merkle_proof,
    },
};
use risc0_zkvm::guest::env;
use step_iso::aggregate_pubkey;
use step_iso::types::{SyncStepArgs, SyncStepCircuitOutput};

fn main() {
    let args: SyncStepArgs = env::read();
    verify_merkle_proof(
        args.execution_payload_branch.to_vec(),
        args.execution_payload_root,
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
        args.finality_branch,
        finalized_header_root,
        &args.attested_header.state_root.to_vec(),
        105,
    );
    /*env::commit(&CommitteeCircuitOutput::new(
        finalized_state_root,
        commitment,
        finalized_header_root,
    ));*/
}
