#[cfg(not(feature = "sp1"))]
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    pairing, G1Affine, G1Projective, G2Affine, G2Projective,
};
#[cfg(feature = "sp1")]
use bls12_381_sp1::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    pairing, G1Affine, G1Projective, G2Affine, G2Projective,
};
use committee_iso::utils::{
    add_left_right, commit_to_keys, compute_digest, decode_pubkeys_x, merkleize_keys,
    uint64_to_le_256, Sha256,
};
use types::Commitment;
use types::SyncStepArgs;

pub mod types;
pub mod utils;

fn aggregate_pubkey(args: SyncStepArgs) -> (G1Affine, Commitment) {
    let pubkey_affines: Vec<G1Affine> = args
        .pubkeys_uncompressed
        .as_slice()
        .iter()
        .map(|bytes| {
            G1Affine::from_uncompressed_unchecked(&bytes.as_slice().try_into().unwrap()).unwrap()
        })
        .collect();

    let pubkeys_compressed: Vec<Vec<u8>> = pubkey_affines
        .iter()
        .map(|uncompressed| uncompressed.to_compressed().to_vec())
        .collect();

    let pubkeys_decoded = decode_pubkeys_x(pubkeys_compressed);
    let pubkey_commitment: Commitment = commit_to_keys(pubkeys_decoded.0, pubkeys_decoded.1);

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

    (generator.into(), pubkey_commitment)
}

pub fn verify_aggregate_signature(args: SyncStepArgs, committee_commitment: [u8; 32]) {
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let (aggregate_key, commitment): (G1Affine, Commitment) = aggregate_pubkey(args.clone());
    assert_eq!(commitment, committee_commitment);
    let attested_header_root = merkleize_keys(vec![
        uint64_to_le_256(args.attested_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(args.attested_header.proposer_index.parse::<u64>().unwrap()),
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
}

#[cfg(test)]
mod tests {
    use crate::{utils::load_circuit_args_env, verify_aggregate_signature};
    use committee_iso::utils::{merkleize_keys, uint64_to_le_256, verify_merkle_proof};

    #[test]
    fn test_aggregate_pubkey_commitment_and_verify_signature() {
        let args = load_circuit_args_env();
        let commitment: [u8; 32] = [
            105, 137, 53, 187, 214, 9, 37, 142, 162, 195, 216, 252, 41, 0, 37, 135, 102, 197, 110,
            192, 183, 234, 101, 253, 40, 247, 143, 101, 172, 85, 164, 105,
        ];
        verify_aggregate_signature(args.clone(), commitment);
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

        // equivalent to a block hash
        let finalized_header_root = merkleize_keys(vec![
            uint64_to_le_256(args.finalized_header.slot.parse::<u64>().unwrap()),
            uint64_to_le_256(args.finalized_header.proposer_index.parse::<u64>().unwrap()),
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
