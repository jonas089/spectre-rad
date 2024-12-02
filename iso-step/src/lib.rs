//#[cfg(all(not(feature = "sp1"), not(feature = "risc0")))]
use bls12_381::{
    hash_to_curve::{self, ExpandMsgXmd, HashToCurve, Message},
    pairing, G1Affine, G1Projective, G2Affine, G2Projective,
};
/*#[cfg(all(feature = "sp1"))]
use bls12_381_sp1::{
    hash_to_curve,
    hash_to_curve::{ExpandMessage, ExpandMsgXmd, HashToCurve, InitExpandMessage},
    pairing, G1Affine, G1Projective, G2Affine, G2Projective,
};*/
use committee_iso::utils::{
    add_left_right, compute_digest, merkleize_keys, uint64_to_le_256, Sha256,
};
use types::Commitment;
use types::SyncStepArgs;

pub mod types;
pub mod utils;

fn aggregate_pubkey(args: SyncStepArgs) -> G1Affine {
    // performance overhead
    let pubkey_affines: Vec<G1Affine> = args
        .pubkeys_uncompressed
        .as_slice()
        .iter()
        .map(|bytes| {
            G1Affine::from_uncompressed_unchecked(&bytes.as_slice().try_into().unwrap()).unwrap()
        })
        .collect();

    let mut generator = G1Projective::identity();
    let participation_bits = args.pariticipation_bits;
    for (affine, bits) in itertools::multizip((pubkey_affines, participation_bits)) {
        let affine_projective = G1Projective::from(affine);
        if !bits {
            continue;
        }
        // double if equal, add if unequal
        if generator == affine_projective {
            // double
            generator = G1Projective::from(generator).double().into();
        } else {
            // add
            generator = (generator + G1Projective::from(affine)).into();
        }
    }

    generator.into()
}

pub fn verify_aggregate_signature(args: SyncStepArgs) -> Commitment {
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let aggregate_key: G1Affine = aggregate_pubkey(args.clone());
    let attested_header_root = merkleize_keys(vec![
        uint64_to_le_256(args.attested_header.slot),
        uint64_to_le_256(args.attested_header.proposer_index as u64),
        args.attested_header.parent_root.to_vec(),
        args.attested_header.state_root.to_vec(),
        args.attested_header.body_root.to_vec(),
    ]);

    let signing_root: Vec<u8> = add_left_right(attested_header_root, &args.domain.to_vec());
    let message_g2: G2Projective =
        <G2Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(
            [compute_digest(&signing_root)],
            DST,
        );
    let signature: G2Affine =
        G2Affine::from_compressed_unchecked(&args.signature_compressed.try_into().unwrap())
            .unwrap();

    // e(hash_msg,pub_key)=e(signature,g1)
    assert_eq!(
        pairing(&aggregate_key, &message_g2.into()),
        pairing(&G1Affine::generator(), &signature)
    );

    // return commitment to aggregate pubkey
    compute_digest(&aggregate_key.to_compressed().to_vec())
}

#[cfg(test)]
mod tests {
    use crate::{utils::load_circuit_args_env, verify_aggregate_signature};
    use committee_iso::utils::{merkleize_keys, uint64_to_le_256, verify_merkle_proof};

    #[test]
    fn test_aggregate_pubkey_commitment_and_verify_signature() {
        let args = load_circuit_args_env();
        verify_aggregate_signature(args.clone());
    }

    #[test]
    fn test_verify_roots() {
        let args: crate::types::SyncStepArgs = load_circuit_args_env();

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
    }
}
