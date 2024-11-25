use bls12_381::{G1Affine, G1Projective};
use types::SyncStepArgs;

pub mod types;
pub mod utils;

/*
1. aggregate pubkeys into one
2. message hash is hash of signing root
3. bls.assert_valid_signature(signature, msghash, agg_pubkey);
4. compute finalized_header_root & verify merkle proof
5. compute finalized_block_body_root & verify merkle proof
*/

pub fn aggregate_pubkey(args: SyncStepArgs) -> G1Affine {
    let pubkey_affines: Vec<G1Affine> = args
        .pubkeys_uncompressed
        .as_slice()
        .iter()
        .map(|bytes| G1Affine::from_uncompressed(&bytes.as_slice().try_into().unwrap()).unwrap())
        .collect();
    let mut generator = G1Affine::generator();
    let participation_bits = args.pariticipation_bits;
    for (affine, bits) in itertools::multizip((pubkey_affines, participation_bits)) {
        if !bits {
            continue;
        }
        // double if equal, add if unequal
        if generator == affine {
            // double
            generator = (generator + G1Projective::from(affine)).into();
        } else {
            // add
            generator = G1Projective::from(generator).double().into();
        }
    }
    println!("Aggregate Key: {:?}", &generator);
    generator
}

#[cfg(test)]
mod tests {
    use committee_iso::utils::{digest, merkleize_keys, verify_merkle_proof};

    use crate::{aggregate_pubkey, utils::load_circuit_args_env};
    #[test]
    fn test_aggregate_pubkey_commitment() {
        let args = load_circuit_args_env();
        let aggregated_pubkey = aggregate_pubkey(args.clone());
        let commitment = digest(&aggregated_pubkey.to_compressed().to_vec());
        let signature_bytes = args.signature_compressed;
        println!("Aggregate commitment: {:?}", &commitment);

        todo!("Verify the signature against the aggregated key!")
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

        let attested_header_root = merkleize_keys(vec![
            args.attested_header.slot.to_be_bytes().to_vec(),
            args.attested_header.proposer_index.to_be_bytes().to_vec(),
            args.attested_header.parent_root.to_vec(),
            args.attested_header.state_root.to_vec(),
            args.attested_header.body_root.to_vec(),
        ]);

        let finalized_header_root = merkleize_keys(vec![
            args.finalized_header.slot.to_be_bytes().to_vec(),
            args.finalized_header.proposer_index.to_be_bytes().to_vec(),
            args.finalized_header.parent_root.to_vec(),
            args.finalized_header.state_root.to_vec(),
            args.finalized_header.body_root.to_vec(),
        ]);

        verify_merkle_proof(
            args.finality_branch,
            finalized_header_root,
            &attested_header_root,
            105,
        );
    }
}
