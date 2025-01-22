#[cfg(test)]
mod test_circuits {
    use committee_iso::{
        types::{CommitteeCircuitOutput, CommitteeUpdateArgs},
        utils::load_circuit_args_env as load_committee_args_env,
    };
    use prover::{
        fixture::{create_committee_proof_fixture, create_step_proof_fixture},
        generate_committee_update_proof_sp1, generate_step_proof_sp1, ProverOps,
    };
    use step_iso::{types::SyncStepArgs, utils::load_circuit_args_env as load_step_args_env};

    #[test]
    fn test_committee_circuit_default_sp1() {
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let (proof, _) = generate_committee_update_proof_sp1(
            &ProverOps::Default,
            committee_update,
            &prover::ProofCompressionBool::Uncompressed,
        );
        let output: CommitteeCircuitOutput =
            borsh::from_slice(&proof.public_values.as_slice()).unwrap();
        println!("Output: {:?}", &output);
    }

    // SP1 Committee Circuit Wrapped
    #[test]
    fn test_committee_circuit_groth16_sp1() {
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let ops = ProverOps::Groth16;
        let (proof, vk) = generate_committee_update_proof_sp1(
            &ops,
            committee_update,
            &prover::ProofCompressionBool::Uncompressed,
        );
        create_committee_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_committee_circuit_plonk_sp1() {
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let ops = ProverOps::Plonk;
        let (proof, vk) = generate_committee_update_proof_sp1(
            &ops,
            committee_update,
            &prover::ProofCompressionBool::Uncompressed,
        );
        create_committee_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_step_circuit_default_sp1() {
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let commitment: [u8; 32] = [
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
        ];
        generate_step_proof_sp1(
            &ProverOps::Default,
            commitment,
            sync_step_args,
            &prover::ProofCompressionBool::Uncompressed,
        );
    }

    #[test]
    fn test_step_circuit_groth16_sp1() {
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let ops = ProverOps::Groth16;
        let commitment: [u8; 32] = [
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
        ];
        let (proof, vk) = generate_step_proof_sp1(
            &ops,
            commitment,
            sync_step_args,
            &prover::ProofCompressionBool::Uncompressed,
        );
        create_step_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_step_circuit_plonk_sp1() {
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let ops = ProverOps::Plonk;
        let commitment: [u8; 32] = [
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
        ];
        let (proof, vk) = generate_step_proof_sp1(
            &ops,
            commitment,
            sync_step_args,
            &prover::ProofCompressionBool::Uncompressed,
        );
        create_step_proof_fixture(&proof, &vk, &ops);
    }
}
