use committee_iso::{
    types::{CommitteeUpdateArgs, PublicKeyHashes, Root},
    utils::{hash_keys, merkleize_keys, verify_merkle_proof},
};
use risc0_zkvm::guest::env;
fn main() {
    let args: CommitteeUpdateArgs = env::read();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
    let committee_root_ssz: Root = merkleize_keys(key_hashs);
    let finalized_state_root: Root = args.finalized_header.state_root.to_vec();

    verify_merkle_proof(
        args.sync_committee_branch,
        committee_root_ssz,
        &finalized_state_root,
        110,
    );

    env::commit(&finalized_state_root);
}
