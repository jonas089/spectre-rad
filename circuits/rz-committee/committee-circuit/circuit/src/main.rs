use committee_iso::{
    types::{CommitteeCircuitInput, PublicKeyHashes, Root},
    utils::{hash_keys, merkleize_keys, verify_merkle_proof},
};
use risc0_zkvm::guest::env;
fn main() {
    let args: CommitteeCircuitInput = env::read();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys.clone());
    let committee_root_ssz: Root = merkleize_keys(key_hashs);
    let finalized_state_root: Root = args.state_root;
    verify_merkle_proof(args.branch, committee_root_ssz, &finalized_state_root, 110);
    env::commit(&finalized_state_root);
}
