use committee_iso::{
    types::{CommitteeCircuitOutput, CommitteeUpdateArgs, PublicKeyHashes},
    utils::{
        commit_to_keys, decode_pubkeys_x, hash_keys, merkleize_keys, uint64_to_le_256,
        verify_merkle_proof,
    },
};
use risc0_zkvm::guest::env;
use std::io::Read;

fn main() {
    let mut buffer: Vec<u8> = vec![];
    let _ = env::stdin().read_to_end(&mut buffer);
    let args: CommitteeUpdateArgs = borsh::from_slice(&buffer).unwrap();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
    let committee_root_ssz: Vec<u8> = merkleize_keys(key_hashs);
    let finalized_state_root: Vec<u8> = args.finalized_header.state_root.to_vec();
    let (keys, signs) = decode_pubkeys_x(args.pubkeys_compressed);
    let commitment = commit_to_keys(keys);

    verify_merkle_proof(
        args.sync_committee_branch,
        committee_root_ssz,
        &finalized_state_root,
        110,
    );

    let finalized_header_root: Vec<u8> = merkleize_keys(vec![
        uint64_to_le_256(args.finalized_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(args.finalized_header.proposer_index.parse::<u64>().unwrap()),
        args.finalized_header.parent_root.to_vec(),
        finalized_state_root.clone(),
        args.finalized_header.body_root.to_vec(),
    ]);

    env::commit(&CommitteeCircuitOutput::new(
        commitment.try_into().unwrap(),
        finalized_header_root.try_into().unwrap(),
    ));
}
