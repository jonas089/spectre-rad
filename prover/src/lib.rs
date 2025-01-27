pub mod eth;
pub mod fixture;
use committee_iso::types::CommitteeUpdateArgs;
use rotation_iso::types::RotationCircuitInputs;
use sp1_sdk::{include_elf, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use step_iso::types::{SyncStepArgs, SyncStepCircuitInput};
pub enum ProofCompressionBool {
    Compressed,
    Uncompressed,
}

pub enum ProverOps {
    Default,
    Groth16,
    Plonk,
}

pub fn generate_committee_update_proof_sp1(
    ops: &ProverOps,
    committee_update: CommitteeUpdateArgs,
    compressed: &ProofCompressionBool,
) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
    use std::time::Instant;
    let start_time = Instant::now();
    #[allow(deprecated)]
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(borsh::to_vec(&committee_update).expect("Failed to serialize"));

    let (proof, _, vk) = match ops {
        ProverOps::Default => {
            const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
            let (pk, vk) = client.setup(COMMITTEE_ELF);
            let proof = match compressed {
                ProofCompressionBool::Compressed => client
                    .prove(&pk, &stdin)
                    .compressed()
                    .run()
                    .expect("failed to generate proof"),
                ProofCompressionBool::Uncompressed => client
                    .prove(&pk, &stdin)
                    .run()
                    .expect("failed to generate proof"),
            };
            (proof, pk, vk)
        }
        ProverOps::Groth16 => {
            const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
            let (pk, vk) = client.setup(COMMITTEE_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
        ProverOps::Plonk => {
            const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
            let (pk, vk) = client.setup(COMMITTEE_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .plonk()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
    };
    println!("Successfully generated proof!");
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
    (proof, vk)
}

pub fn generate_step_proof_sp1(
    ops: &ProverOps,
    commitment: [u8; 32],
    sync_step_args: SyncStepArgs,
    compressed: &ProofCompressionBool,
) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
    use std::time::Instant;
    sp1_sdk::utils::setup_logger();
    let start_time = Instant::now();
    let inputs: SyncStepCircuitInput = SyncStepCircuitInput {
        args: sync_step_args,
        commitment,
    };
    #[allow(deprecated)]
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(borsh::to_vec(&inputs).expect("Failed to serialize"));

    let (proof, _, vk) = match ops {
        ProverOps::Default => {
            const STEP_ELF: &[u8] = include_elf!("sp1-step");
            let (pk, vk) = client.setup(STEP_ELF);
            let proof = match compressed {
                ProofCompressionBool::Compressed => client
                    .prove(&pk, &stdin)
                    .compressed()
                    .run()
                    .expect("failed to generate proof"),
                ProofCompressionBool::Uncompressed => client
                    .prove(&pk, &stdin)
                    .run()
                    .expect("failed to generate proof"),
            };
            (proof, pk, vk)
        }
        ProverOps::Groth16 => {
            const STEP_ELF: &[u8] = include_elf!("sp1-step");
            let (pk, vk) = client.setup(STEP_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
        ProverOps::Plonk => {
            const STEP_ELF: &[u8] = include_elf!("sp1-step");
            let (pk, vk) = client.setup(STEP_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .plonk()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
    };
    println!("Successfully generated proof!");
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
    (proof, vk)
}

pub fn generate_rotation_proof_sp1(
    ops: &ProverOps,
    inputs: RotationCircuitInputs,
) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
    use std::time::Instant;
    //sp1_sdk::utils::setup_logger();
    let start_time = Instant::now();
    #[allow(deprecated)]
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(borsh::to_vec(&inputs).expect("Failed to serialize"));
    let (proof, _, vk) = match ops {
        ProverOps::Default => {
            panic!("Recursive Step Proof for Default Prover mode is not supported!")
        }
        ProverOps::Groth16 => {
            const RECURSIVE_ELF: &[u8] = include_elf!("sp1-rotation");
            let (pk, vk) = client.setup(RECURSIVE_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
        ProverOps::Plonk => {
            const RECURSIVE_ELF: &[u8] = include_elf!("sp1-rotation");
            let (pk, vk) = client.setup(RECURSIVE_ELF);
            let proof = client
                .prove(&pk, &stdin)
                .plonk()
                .run()
                .expect("failed to generate proof");
            (proof, pk, vk)
        }
    };
    println!("Successfully generated proof!");
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
    (proof, vk)
}
