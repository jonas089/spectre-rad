use bls12_381::{
    hash_to_curve,
    hash_to_curve::HashToCurve,
    hash_to_curve::{ExpandMessage, ExpandMsgXmd},
    G1Affine, G1Projective,
};
use committee_iso::utils::{
    add_left_right, compute_digest, merkleize_keys, uint64_to_le_256, Sha256,
};
use types::Commitment;
use types::SyncStepArgs;
/*#[cfg(all(feature = "sp1", not(feature = "default")))]
use {
    bls12_381_sp1::{
        hash_to_curve,
        hash_to_curve::{ExpandMessage, ExpandMsgXmd, InitExpandMessage},
        G1Affine, G1Projective,
    },
};*/
#[cfg(feature = "blst")]
use {blst::min_pk as bls, blst::BLST_ERROR};

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
    // DATA: public key
    let aggregate_key: G1Affine = aggregate_pubkey(args.clone());
    let commitment = compute_digest(&aggregate_key.to_compressed().to_vec());

    // DATA: signature
    let signature_bytes: Vec<u8> = args.signature_compressed;
    let attested_header_root = merkleize_keys(vec![
        uint64_to_le_256(args.attested_header.slot),
        uint64_to_le_256(args.attested_header.proposer_index as u64),
        args.attested_header.parent_root.to_vec(),
        args.attested_header.state_root.to_vec(),
        args.attested_header.body_root.to_vec(),
    ]);
    let domain = args.domain.to_vec();

    // DATA: message hash -> should be hash to curve bls12_381
    let message_not_on_curve: Vec<u8> = add_left_right(attested_header_root, &domain);
    let message_g1 = <G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::encode_to_curve(
        [message_not_on_curve],
        DST,
    );
    // Prepare a buffer to hold the expanded bytes
    /*let mut output = vec![0u8; 48];
    expander.read_into(&mut output);*/

    //let signing_root: Vec<u8> = digest(&add_left_right(attested_header_root, &domain));

    #[cfg(feature = "blst")]
    {
        let signature = bls::Signature::from_bytes(&signature_bytes).unwrap();
        let public_key = bls::PublicKey::deserialize(&aggregate_key.to_compressed()).unwrap();
        let res = signature.verify(true, &signing_root, DST, &[], &public_key, true);
        // revert if signature is invalid
        assert_eq!(res, BLST_ERROR::BLST_SUCCESS);
    }
    // return the aggregate key commitment
    commitment
}

#[cfg(test)]
mod tests {
    use crate::{aggregate_pubkey, utils::load_circuit_args_env};
    use committee_iso::utils::{
        add_left_right, compute_digest, merkleize_keys, uint64_to_le_256, verify_merkle_proof,
    };
    #[cfg(feature = "blst")]
    use {blst::min_pk as bls, blst::BLST_ERROR};

    #[test]
    fn test_aggregate_pubkey_commitment_and_verify_signature() {
        const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
        let args = load_circuit_args_env();
        let aggregated_pubkey = aggregate_pubkey(args.clone());
        let commitment = compute_digest(&aggregated_pubkey.to_compressed().to_vec());
        let signature_bytes = args.signature_compressed;
        println!("Aggregate commitment: {:?}", &commitment);
        let attested_header_root = merkleize_keys(vec![
            uint64_to_le_256(args.attested_header.slot),
            uint64_to_le_256(args.attested_header.proposer_index as u64),
            args.attested_header.parent_root.to_vec(),
            args.attested_header.state_root.to_vec(),
            args.attested_header.body_root.to_vec(),
        ]);
        let domain = args.domain.to_vec();
        let signing_root = compute_digest(&add_left_right(attested_header_root, &domain));
        #[cfg(feature = "blst")]
        {
            let signature = bls::Signature::from_bytes(&signature_bytes).unwrap();
            let public_key =
                bls::PublicKey::deserialize(&aggregated_pubkey.to_compressed()).unwrap();
            let res = signature.verify(true, &signing_root, DST, &[], &public_key, true);
            assert_eq!(res, BLST_ERROR::BLST_SUCCESS);
        }
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
