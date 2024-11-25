use committee_iso::{
    types::{CommitteeCircuitInput, CommitteeCircuitOutput, PublicKeyHashes, Root},
    utils::{
        decode_pubkeys_x, hash_keys, merkleize_keys, precompile_sha2_commitment, uint64_to_le_256,
        verify_merkle_proof,
    },
};
use risc0_zkvm::guest::env;
use step_iso::aggregate_pubkey;

fn main() {
    /*let args: CommitteeCircuitInput = env::read();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys.clone());
    let committee_root_ssz: Root = merkleize_keys(key_hashs);
    let finalized_state_root: Root = args.finalized_header.state_root.to_vec();
    let (keys, signs) = decode_pubkeys_x(args.pubkeys);
    let commitment = precompile_sha2_commitment(keys, signs);
    // todo: compute and commit the finalized_header_root!
    let finalized_header_root: Vec<u8> = merkleize_keys(vec![
        uint64_to_le_256(args.finalized_header.slot as u64),
        uint64_to_le_256(args.finalized_header.proposer_index as u64),
        args.finalized_header.parent_root.to_vec(),
        finalized_state_root.clone(),
        args.finalized_header.body_root.to_vec(),
    ]);
    verify_merkle_proof(args.branch, committee_root_ssz, &finalized_state_root, 110);
    env::commit(&CommitteeCircuitOutput::new(
        finalized_state_root,
        commitment,
        finalized_header_root,
    ));*/
}
