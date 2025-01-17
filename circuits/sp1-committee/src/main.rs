#![no_main]
use committee_iso::{
    types::{CommitteeCircuitOutput, CommitteeUpdateArgs, PublicKeyHashes},
    utils::{
        commit_to_keys_with_sign, decode_pubkeys_x, hash_keys, merkleize_keys, uint64_to_le_256,
        verify_merkle_proof,
    },
};
#[cfg(feature = "wrapped")]
use ::{
    alloy_primitives::FixedBytes, alloy_sol_types::SolType, committee_iso::types::WrappedOutput,
};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let args: CommitteeUpdateArgs = borsh::from_slice(&sp1_zkvm::io::read_vec()).unwrap();
    let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
    let committee_root_ssz: Vec<u8> = merkleize_keys(key_hashs);
    let finalized_state_root: Vec<u8> = args.finalized_header.state_root.to_vec();
    let (keys, signs) = decode_pubkeys_x(args.pubkeys_compressed);
    let commitment = commit_to_keys_with_sign(&keys, &signs);
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
    #[cfg(not(feature = "wrapped"))]
    sp1_zkvm::io::commit_slice(
        &borsh::to_vec(&CommitteeCircuitOutput::new(
            commitment,
            finalized_header_root.try_into().unwrap(),
        ))
        .unwrap(),
    );
    #[cfg(feature = "wrapped")]
    {
        let bytes = WrappedOutput::abi_encode(&WrappedOutput {
            commitment: FixedBytes::<32>::from_slice(&commitment),
            finalized_header_root: FixedBytes::<32>::from_slice(&finalized_header_root),
        });

        sp1_zkvm::io::commit_slice(&bytes);
    }
}
