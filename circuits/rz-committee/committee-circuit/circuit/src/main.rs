use committee_iso::{
    types::{CommitteeCircuitInput, CommitteeCircuitOutput, PublicKeyHashes, Root},
    utils::{
        decode_pubkeys_x, hash_keys, merkleize_keys, poseidon_commit_pubkeys_compressed,
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
    let commitment = poseidon_commit_pubkeys_compressed(keys, signs);

    verify_merkle_proof(args.branch, committee_root_ssz, &finalized_state_root, 110);
    env::commit(&CommitteeCircuitOutput::new(
        finalized_state_root,
        commitment,
    ));
}
