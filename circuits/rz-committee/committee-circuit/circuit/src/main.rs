use committee_iso::{
    types::{CommitteeCircuitOutput, CommitteeUpdateArgs, PublicKeyHashes, Root},
    utils::{
        commit_to_keys, decode_pubkeys_x, hash_keys, merkleize_keys, uint64_to_le_256,
        verify_merkle_proof,
    },
};
use risc0_zkvm::guest::env;

fn main() {
    let args: CommitteeUpdateArgs = env::read();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
    let committee_root_ssz: Root = merkleize_keys(key_hashs);
    let finalized_state_root: Root = args.finalized_header.state_root.to_vec();
    let (keys, signs) = decode_pubkeys_x(args.pubkeys_compressed);
    let commitment = commit_to_keys(keys, signs);
    // todo: compute and commit the finalized_header_root!
    let finalized_header_root: Vec<u8> = merkleize_keys(vec![
        uint64_to_le_256(args.finalized_header.slot as u64),
        uint64_to_le_256(args.finalized_header.proposer_index as u64),
        args.finalized_header.parent_root.to_vec(),
        finalized_state_root.clone(),
        args.finalized_header.body_root.to_vec(),
    ]);
    verify_merkle_proof(
        args.sync_committee_branch,
        committee_root_ssz,
        &finalized_state_root,
        110,
    );
    env::commit(&CommitteeCircuitOutput::new(
        finalized_state_root,
        commitment,
        finalized_header_root,
    ));
}
