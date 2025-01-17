#[cfg(test)]
mod test_circuits {
    use alloy_sol_types::SolType;
    use committee_circuit::{RZ_COMMITTEE_ELF, RZ_COMMITTEE_ID};
    use committee_iso::{
        types::{CommitteeCircuitOutput, CommitteeUpdateArgs, WrappedOutput},
        utils::load_circuit_args_env as load_committee_args_env,
    };
    use prover::{generate_committee_update_proof_sp1, generate_step_proof_sp1, ProverOps};
    use risc0_zkvm::{default_prover, ExecutorEnv};
    use serde::{Deserialize, Serialize};
    use sp1_sdk::{HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};
    use std::path::PathBuf;
    use step_circuit::{RZ_STEP_ELF, RZ_STEP_ID};
    use step_iso::{
        types::{SyncStepArgs, SyncStepCircuitInput},
        utils::load_circuit_args_env as load_step_args_env,
    };

    // Risc0 Committee Circuit
    #[test]
    fn test_committee_circuit_risc0() {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let env = ExecutorEnv::builder()
            .write_slice(&borsh::to_vec(&committee_update).unwrap())
            .build()
            .unwrap();

        let prover = default_prover();
        let prove_info = prover.prove(env, RZ_COMMITTEE_ELF).unwrap();
        let receipt = prove_info.receipt;
        let output: CommitteeCircuitOutput = receipt.journal.decode().unwrap();
        receipt.verify(RZ_COMMITTEE_ID).unwrap();
        println!("Public output: {:?}", &output);
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
    }

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
        create_proof_fixture(&proof, &vk, &ops);
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
        create_proof_fixture(&proof, &vk, &ops);
    }

    // Risc0 Step Circuit
    #[test]
    fn test_step_circuit_risc0() {
        use std::time::Instant;
        let start_time = Instant::now();
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let commitment: [u8; 32] = [
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
        ];
        let inputs: SyncStepCircuitInput = SyncStepCircuitInput {
            args: sync_step_args,
            commitment,
        };
        let env = ExecutorEnv::builder()
            .write_slice(&borsh::to_vec(&inputs).unwrap())
            .build()
            .unwrap();
        let prover = default_prover();
        let prove_info = prover.prove(env, RZ_STEP_ELF).unwrap();
        let receipt = prove_info.receipt;
        receipt.verify(RZ_STEP_ID).unwrap();
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
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
        create_proof_fixture(&proof, &vk, &ops);
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
        create_proof_fixture(&proof, &vk, &ops);
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ProofFixture {
        root: String,
        commitment: String,
        vkey: String,
        public_values: String,
        proof: String,
    }
    fn create_proof_fixture(
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
        ops: &ProverOps,
    ) {
        let bytes = proof.public_values.as_slice();
        let WrappedOutput {
            finalized_header_root,
            commitment,
        } = WrappedOutput::abi_decode(bytes, false).unwrap();
        let fixture = ProofFixture {
            root: format!("0x{}", hex::encode(finalized_header_root)),
            commitment: format!("0x{}", hex::encode(commitment)),
            vkey: vk.bytes32().to_string(),
            public_values: format!("0x{}", hex::encode(bytes)),
            proof: format!("0x{}", hex::encode(proof.bytes())),
        };
        let prefix = match ops {
            ProverOps::Default => panic!("No point in generating a fixture for a default proof!"),
            ProverOps::Groth16 => "groth16-fixture.json",
            ProverOps::Plonk => "plonk-fixture.json",
        };
        let fixture_path = PathBuf::from("./");
        std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
        std::fs::write(
            fixture_path.join(prefix),
            serde_json::to_string_pretty(&fixture).unwrap(),
        )
        .expect("failed to write fixture");
    }
}
