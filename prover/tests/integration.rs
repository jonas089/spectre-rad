#[cfg(test)]
mod tests {
    use aggregate_iso::types::RecursiveInputs;
    use committee_iso::utils::{commit_to_keys_with_sign, decode_pubkeys_x};
    use preprocessor::get_light_client_update;
    use prover::{
        generate_aggregate_proof_sp1, generate_committee_update_proof_sp1, generate_step_proof_sp1,
    };
    use sp1_sdk::HashableKey;
    use std::path::Path;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_committee_update_beacon_cli_e2e() {
        let path = Path::new("/Users/chef/.sp1/circuits/plonk/v3.0.0");
        if tokio::fs::metadata(path).await.is_ok() {
            tokio::fs::remove_dir_all(path)
                .await
                .expect("Failed to remove directory");
        }

        let ((s, c), oc) = get_light_client_update().await;

        let (committee_proof, committee_vk) = generate_committee_update_proof_sp1(
            &prover::ProverOps::Default,
            c,
            &prover::ProofCompressionBool::Compressed,
        );
        let committee_public_values = committee_proof.public_values.to_vec();
        let committee_inputs = RecursiveInputs {
            public_values: committee_public_values,
            vk: committee_vk.hash_u32(),
        };

        let pubkeys_x_decoded = decode_pubkeys_x(oc.clone());
        let commitment = commit_to_keys_with_sign(&pubkeys_x_decoded.0, &pubkeys_x_decoded.1);

        let (step_proof, step_vk) = generate_step_proof_sp1(
            &prover::ProverOps::Default,
            commitment,
            s,
            &prover::ProofCompressionBool::Compressed,
        );
        let step_public_values = step_proof.public_values.to_vec();
        let step_inputs = RecursiveInputs {
            public_values: step_public_values,
            vk: step_vk.hash_u32(),
        };
        let (_proof, _vk) = generate_aggregate_proof_sp1(
            &prover::ProverOps::Plonk,
            vec![committee_inputs, step_inputs],
            vec![committee_proof, step_proof],
            vec![committee_vk, step_vk],
        );
    }
}
