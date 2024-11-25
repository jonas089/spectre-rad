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
    generator
}

#[cfg(test)]
mod tests {
    use committee_iso::utils::digest;

    use crate::{aggregate_pubkey, utils::load_circuit_args_env};
    #[test]
    fn test_aggregate_pubkey_commitment() {
        let args = load_circuit_args_env();
        let aggregated_pubkey = aggregate_pubkey(args);
        let commitment = digest(&aggregated_pubkey.to_compressed().to_vec());
        println!("Aggregate commitment: {:?}", &commitment);
    }
}
