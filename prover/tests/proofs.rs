#[cfg(test)]
mod test_circuits {
    use alloy_sol_types::SolType;
    use committee_circuit::{RZ_COMMITTEE_ELF, RZ_COMMITTEE_ID};
    use committee_iso::{
        types::{CommitteeCircuitOutput, CommitteeUpdateArgs, WrappedOutput},
        utils::load_circuit_args_env as load_committee_args_env,
    };
    use prover::ProverOps;
    use risc0_zkvm::{default_prover, ExecutorEnv};
    use serde::{Deserialize, Serialize};
    use sp1_sdk::{
        include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
    };
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

    fn test_committee_circuit_sp1(ops: &ProverOps) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let client = ProverClient::new();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(borsh::to_vec(&committee_update).expect("Failed to serialize"));

        let (proof, _, vk) = match ops {
            ProverOps::Default => {
                const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
                let (pk, vk) = client.setup(COMMITTEE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
            ProverOps::Groth16 => {
                const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
                let (pk, vk) = client.setup(COMMITTEE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .groth16()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
            ProverOps::Plonk => {
                const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
                let (pk, vk) = client.setup(COMMITTEE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .plonk()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
        };
        println!("Successfully generated proof!");
        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
        (proof, vk)
    }

    #[test]
    fn test_committee_circuit_default_sp1() {
        test_committee_circuit_sp1(&ProverOps::Default);
    }

    // SP1 Committee Circuit Wrapped
    #[test]
    fn test_committee_circuit_groth16_sp1() {
        let ops = ProverOps::Groth16;
        let (proof, vk) = test_committee_circuit_sp1(&ops);
        create_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_committee_circuit_plonk_sp1() {
        let ops = ProverOps::Plonk;
        let (proof, vk) = test_committee_circuit_sp1(&ops);
        create_proof_fixture(&proof, &vk, &ops);
    }

    // Risc0 Step Circuit
    #[test]
    fn test_step_circuit_risc0() {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let commitment: [u8; 32] = [
            105, 137, 53, 187, 214, 9, 37, 142, 162, 195, 216, 252, 41, 0, 37, 135, 102, 197, 110,
            192, 183, 234, 101, 253, 40, 247, 143, 101, 172, 85, 164, 105,
        ];
        let inputs: SyncStepCircuitInput = SyncStepCircuitInput {
            args: sync_step_args,
            committee_commitment: commitment,
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

    // SP1 Step Circuit
    fn test_step_circuit_sp1(ops: &ProverOps) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
        use std::time::Instant;
        sp1_sdk::utils::setup_logger();
        let start_time = Instant::now();
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let commitment: [u8; 32] = [
            105, 137, 53, 187, 214, 9, 37, 142, 162, 195, 216, 252, 41, 0, 37, 135, 102, 197, 110,
            192, 183, 234, 101, 253, 40, 247, 143, 101, 172, 85, 164, 105,
        ];
        let inputs: SyncStepCircuitInput = SyncStepCircuitInput {
            args: sync_step_args,
            committee_commitment: commitment,
        };
        let client = ProverClient::new();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(borsh::to_vec(&inputs).expect("Failed to serialize"));

        let (proof, _, vk) = match ops {
            ProverOps::Default => {
                const STEP_ELF: &[u8] = include_elf!("sp1-step");
                let (pk, vk) = client.setup(STEP_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
            ProverOps::Groth16 => {
                const STEP_ELF: &[u8] = include_elf!("sp1-step");
                let (pk, vk) = client.setup(STEP_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .groth16()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
            ProverOps::Plonk => {
                const STEP_ELF: &[u8] = include_elf!("sp1-step");
                let (pk, vk) = client.setup(STEP_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .plonk()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
        };
        println!("Successfully generated proof!");
        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
        (proof, vk)
    }

    #[test]
    fn test_step_circuit_default_sp1() {
        test_step_circuit_sp1(&ProverOps::Default);
    }

    #[test]
    fn test_step_circuit_groth16_sp1() {
        let ops = ProverOps::Groth16;
        let (proof, vk) = test_step_circuit_sp1(&ops);
        create_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_step_circuit_plonk_sp1() {
        let ops = ProverOps::Plonk;
        let (proof, vk) = test_step_circuit_sp1(&ops);
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
        let WrappedOutput { root, commitment } = WrappedOutput::abi_decode(bytes, false).unwrap();
        let fixture = ProofFixture {
            root: format!("0x{}", hex::encode(root)),
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
