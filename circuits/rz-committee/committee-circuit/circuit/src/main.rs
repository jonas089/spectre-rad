use committee_iso::{
    types::{CommitteeCircuitInput, CommitteeCircuitOutput, PublicKeyHashes, Root},
    utils::{
        decode_pubkeys_x, hash_keys, merkleize_keys, precompile_sha2_commitment,
        verify_merkle_proof,
    },
};
use risc0_zkvm::guest::env;

fn main() {
    let args: CommitteeCircuitInput = env::read();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys.clone());
    let committee_root_ssz: Root = merkleize_keys(key_hashs);
    let finalized_state_root: Root = args.state_root;
    let (keys, signs) = decode_pubkeys_x(args.pubkeys);
    let commitment = precompile_sha2_commitment(keys, signs);
    // todo: compute and commit the finalized_header_root!
    verify_merkle_proof(args.branch, committee_root_ssz, &finalized_state_root, 110);
    env::commit(&CommitteeCircuitOutput::new(
        finalized_state_root,
        commitment,
    ));
}
