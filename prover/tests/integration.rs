#[cfg(test)]
mod tests {
    use committee_iso::utils::{commit_to_keys_with_sign, decode_pubkeys_x};
    use preprocessor::get_light_client_update_at_slot;
    use prover::{
        fixture::{create_rotation_proof_fixture, create_step_proof_fixture},
        generate_rotation_proof_sp1, generate_step_proof_sp1,
    };
    use rotation_iso::types::RotationCircuitInputs;
    use sp1_sdk::ProverClient;
    use step_iso::types::SyncStepCircuitInput;

    #[tokio::test]
    async fn generate_rotation_proof_payload() {
        let (sc, oc) = get_light_client_update_at_slot(6897664 - (256 * 32)).await;
        let s = sc.clone().unwrap().0;
        let c = sc.unwrap().1;
        let (keys, signs) = decode_pubkeys_x(oc.unwrap());
        let commitment = commit_to_keys_with_sign(&keys, &signs);
        let rotation_inputs = RotationCircuitInputs {
            committee: c,
            step: SyncStepCircuitInput {
                args: s,
                commitment: commitment,
            },
        };
        let (rotation_proof, rotation_vk) = tokio::task::spawn_blocking(move || {
            generate_rotation_proof_sp1(&prover::ProverOps::Groth16, rotation_inputs)
        })
        .await
        .expect("Failed to join the spawned blocking task");
        #[allow(deprecated)]
        let client = ProverClient::new();
        client
            .verify(&rotation_proof, &rotation_vk)
            .expect("Failed to verify rotation proof");
        create_rotation_proof_fixture(&rotation_proof, &rotation_vk, &prover::ProverOps::Groth16);
    }

    #[tokio::test]
    async fn find_last_committee() {
        let mut current_height = 6823936;
        loop {
            current_height += 32 * 256;
            let (sc, _) = get_light_client_update_at_slot(current_height).await;
            match sc {
                Some(_) => {
                    println!("Slot: {}", current_height)
                }
                None => {
                    break;
                }
            }
        }
    }

    #[tokio::test]
    async fn generate_step_proof_payload() {
        let (sc, oc) = get_light_client_update_at_slot(6897664 - (256 * 32)).await;
        let s = sc.clone().unwrap().0;
        let c = sc.unwrap().1;
        let (keys, signs) = decode_pubkeys_x(oc.unwrap());
        let commitment = commit_to_keys_with_sign(&keys, &signs);
        let (step_proof, step_vk) = tokio::task::spawn_blocking(move || {
            generate_step_proof_sp1(
                &prover::ProverOps::Groth16,
                commitment,
                s,
                &prover::ProofCompressionBool::Uncompressed,
            )
        })
        .await
        .expect("Failed to join the spawned blocking task");
        #[allow(deprecated)]
        let client = ProverClient::new();
        client
            .verify(&step_proof, &step_vk)
            .expect("Failed to verify rotation proof");
        create_step_proof_fixture(&step_proof, &step_vk, &prover::ProverOps::Groth16);
    }

    #[tokio::test]
    async fn test_committee_rotation_beacon_cli_e2e_plonk() {
        let (sc, oc) = get_light_client_update_at_slot(6823936).await;
        let s = sc.clone().unwrap().0;
        let c = sc.unwrap().1;
        let (keys, signs) = decode_pubkeys_x(oc.unwrap());
        let commitment = commit_to_keys_with_sign(&keys, &signs);
        let rotation_inputs = RotationCircuitInputs {
            committee: c,
            step: SyncStepCircuitInput {
                args: s,
                commitment: commitment,
            },
        };
        let (rotation_proof, rotation_vk) = tokio::task::spawn_blocking(move || {
            generate_rotation_proof_sp1(&prover::ProverOps::Plonk, rotation_inputs)
        })
        .await
        .expect("Failed to join the spawned blocking task");
        #[allow(deprecated)]
        let client = ProverClient::new();
        client
            .verify(&rotation_proof, &rotation_vk)
            .expect("Failed to verify rotation proof");
    }
}
