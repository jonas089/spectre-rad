pub mod constants;
pub mod types;
pub mod utils;
use types::CommitteeUpdateArgs;

#[cfg(test)]
mod test {
    use crate::{
        types::PublicKeyHashes,
        utils::{
            commit_to_keys, decode_pubkeys_x, hash_keys, load_circuit_args_env, merkleize_keys,
            verify_merkle_proof,
        },
        CommitteeUpdateArgs,
    };
    #[test]
    fn test_verify_committee_root() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
        let committee_root_ssz: Vec<u8> = merkleize_keys(key_hashs);
        let finalized_state_root: Vec<u8> = args.finalized_header.state_root.to_vec();
        verify_merkle_proof(
            args.sync_committee_branch,
            committee_root_ssz,
            &finalized_state_root,
            110,
        );
    }
    #[test]
    fn test_compute_pubkey_commitment() {
        let args: CommitteeUpdateArgs = load_circuit_args_env();
        let keys = decode_pubkeys_x(args.pubkeys_compressed);
        let commitment = commit_to_keys(keys.0, keys.1);
        println!("Commitment: {:?}", &commitment);
    }
}
