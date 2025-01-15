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
        include_elf, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin,
        SP1VerifyingKey,
    };
    use std::path::PathBuf;
    use step_circuit::{RZ_STEP_ELF, RZ_STEP_ID};
    use step_iso::{
        types::{RecursiveInputs, SyncStepArgs, SyncStepCircuitInput},
        utils::load_circuit_args_env as load_step_args_env,
    };

    enum ProofCompressionBool {
        Compressed,
        Uncompressed,
    }

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

    fn test_committee_circuit_sp1(
        ops: &ProverOps,
        compressed: ProofCompressionBool,
    ) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
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
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .compressed()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
            ProverOps::Groth16 => {
                const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
                let (pk, vk) = client.setup(COMMITTEE_ELF);
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .groth16()
                        .compressed()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .groth16()
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
            ProverOps::Plonk => {
                const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
                let (pk, vk) = client.setup(COMMITTEE_ELF);
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .compressed()
                        .plonk()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .plonk()
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
        };
        println!("Successfully generated proof!");
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
        (proof, vk)
    }

    #[test]
    fn test_committee_circuit_default_sp1() {
        let (mut proof, _) =
            test_committee_circuit_sp1(&ProverOps::Default, ProofCompressionBool::Uncompressed);
        let output: CommitteeCircuitOutput = proof.public_values.read::<CommitteeCircuitOutput>();
        println!("Output: {:?}", &output);
    }

    // SP1 Committee Circuit Wrapped
    #[test]
    fn test_committee_circuit_groth16_sp1() {
        let ops = ProverOps::Groth16;
        let (proof, vk) = test_committee_circuit_sp1(&ops, ProofCompressionBool::Uncompressed);
        create_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_committee_circuit_plonk_sp1() {
        let ops = ProverOps::Plonk;
        let (proof, vk) = test_committee_circuit_sp1(&ops, ProofCompressionBool::Uncompressed);
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
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
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
    fn test_step_circuit_sp1(
        ops: &ProverOps,
        compressed: ProofCompressionBool,
    ) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
        use std::time::Instant;
        sp1_sdk::utils::setup_logger();
        let start_time = Instant::now();
        let sync_step_args: SyncStepArgs = load_step_args_env();
        let commitment: [u8; 32] = [
            106, 92, 62, 66, 60, 86, 8, 54, 215, 185, 238, 54, 75, 39, 221, 15, 81, 229, 23, 145,
            198, 242, 244, 199, 60, 103, 60, 206, 116, 216, 86, 227,
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
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .compressed()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
            ProverOps::Groth16 => {
                const STEP_ELF: &[u8] = include_elf!("sp1-step");
                let (pk, vk) = client.setup(STEP_ELF);
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .groth16()
                        .compressed()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .groth16()
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
            ProverOps::Plonk => {
                const STEP_ELF: &[u8] = include_elf!("sp1-step");
                let (pk, vk) = client.setup(STEP_ELF);
                let proof = match compressed {
                    ProofCompressionBool::Compressed => client
                        .prove(&pk, stdin)
                        .compressed()
                        .plonk()
                        .run()
                        .expect("failed to generate proof"),
                    ProofCompressionBool::Uncompressed => client
                        .prove(&pk, stdin)
                        .plonk()
                        .run()
                        .expect("failed to generate proof"),
                };
                (proof, pk, vk)
            }
        };
        println!("Successfully generated proof!");
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
        (proof, vk)
    }

    #[test]
    fn test_step_circuit_default_sp1() {
        test_step_circuit_sp1(&ProverOps::Default, ProofCompressionBool::Uncompressed);
    }

    #[test]
    fn test_step_circuit_groth16_sp1() {
        let ops = ProverOps::Groth16;
        let (proof, vk) = test_step_circuit_sp1(&ops, ProofCompressionBool::Uncompressed);
        create_proof_fixture(&proof, &vk, &ops);
    }

    #[test]
    fn test_step_circuit_plonk_sp1() {
        let ops = ProverOps::Plonk;
        let (proof, vk) = test_step_circuit_sp1(&ops, ProofCompressionBool::Uncompressed);
        create_proof_fixture(&proof, &vk, &ops);
    }

    fn test_step_circuit_sp1_recursive(
        ops: &ProverOps,
        inputs: RecursiveInputs,
        proof: SP1ProofWithPublicValues,
        vk: SP1VerifyingKey,
    ) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
        use std::time::Instant;
        sp1_sdk::utils::setup_logger();
        let start_time = Instant::now();
        let client = ProverClient::new();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(borsh::to_vec(&inputs).expect("Failed to serialize"));
        let SP1Proof::Compressed(proof) = proof.proof else {
            panic!("Uncompressed proof unsupported!")
        };
        stdin.write_proof(*proof, vk.vk);

        let (proof, _, vk) = match ops {
            ProverOps::Default => {
                panic!("Recursive Step Proof for Default Prover mode is not supported!")
            }
            ProverOps::Groth16 => {
                const RECURSIVE_ELF: &[u8] = include_elf!("sp1-recursive-step");
                let (pk, vk) = client.setup(RECURSIVE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .groth16()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
            ProverOps::Plonk => {
                const RECURSIVE_ELF: &[u8] = include_elf!("sp1-recursive-step");
                let (pk, vk) = client.setup(RECURSIVE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .plonk()
                    .run()
                    .expect("failed to generate proof");
                (proof, pk, vk)
            }
        };
        println!("Successfully generated proof!");
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
        (proof, vk)
    }

    #[test]
    // todo: aggregate step and committee proof for commitee update case
    fn test_step_circuit_sp1_recursive_plonk() {
        let (proof, vk) =
            test_step_circuit_sp1(&ProverOps::Default, ProofCompressionBool::Compressed);
        let public_values = proof.public_values.to_vec();
        let inputs = RecursiveInputs {
            public_values,
            vk: vk.hash_u32(),
        };
        let (_proof, _vk) = test_step_circuit_sp1_recursive(&ProverOps::Plonk, inputs, proof, vk);
    }

    #[test]
    // superior to recursive proofs
    // fixed size proof input to the wrapper!
    fn test_step_circuit_sp1_compressed_plonk() {
        let (_proof, _vk) =
            test_step_circuit_sp1(&ProverOps::Plonk, ProofCompressionBool::Compressed);
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
            committee_commitment,
        } = WrappedOutput::abi_decode(bytes, false).unwrap();
        let fixture = ProofFixture {
            root: format!("0x{}", hex::encode(finalized_header_root)),
            commitment: format!("0x{}", hex::encode(committee_commitment)),
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
