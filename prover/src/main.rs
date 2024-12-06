use clap::Parser;
use client::{run, Cli};

mod client;
pub mod integrations;

#[tokio::main]
async fn main() {
    todo!("Update Client to support all circuits!");
    let cli: Cli = Cli::parse();
    run(cli).await;
}

#[cfg(test)]
mod test_risc0 {
    use crate::integrations::aligned::{self, constants::ETH_RPC_URL};
    use committee_circuit::{RZ_COMMITTEE_ELF, RZ_COMMITTEE_ID};
    use committee_iso::{
        types::{CommitteeCircuitOutput, CommitteeUpdateArgs},
        utils::load_circuit_args_env as load_committee_args_env,
    };
    use risc0_zkvm::{default_prover, ExecutorEnv};
    use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
    use step_circuit::{RZ_STEP_ELF, RZ_STEP_ID};
    use step_iso::{types::SyncStepArgs, utils::load_circuit_args_env as load_step_args_env};

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
            .write(&committee_update)
            .unwrap()
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

    // SP1 Committee Circuit
    #[test]
    fn test_committee_circuit_sp1() {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        const COMMITTEE_ELF: &[u8] = include_elf!("sp1-committee");
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let client = ProverClient::new();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(serde_json::to_vec(&committee_update).expect("Failed to serialize"));
        let (pk, vk) = client.setup(COMMITTEE_ELF);
        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");
        println!("Successfully generated proof!");
        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
    }

    // Risc0 Step Circuit
    #[test]
    fn test_step_circuit_risc0() {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        let sync_step: SyncStepArgs = load_step_args_env();
        let env = ExecutorEnv::builder()
            .write(&sync_step)
            .unwrap()
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
    #[test]
    fn test_step_circuit_sp1() {
        use std::time::Instant;
        sp1_sdk::utils::setup_logger();
        let start_time = Instant::now();
        const STEP_ELF: &[u8] = include_elf!("sp1-step");
        let sync_args: SyncStepArgs = load_step_args_env();
        let client = ProverClient::new();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(serde_json::to_vec(&sync_args).expect("Failed to serialize"));
        let (pk, vk) = client.setup(STEP_ELF);
        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");
        println!("Successfully generated proof!");
        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
    }

    // Aligned Layer
    #[tokio::test]
    async fn test_committee_submit_aligned() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let env = ExecutorEnv::builder()
            .write(&committee_update)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();
        let prove_info = prover.prove(env, RZ_COMMITTEE_ELF).unwrap();
        let receipt = prove_info.receipt;
        aligned::submit_committee_proof(
            receipt,
            ETH_RPC_URL,
            17000,
            aligned_sdk::core::types::Network::Holesky,
            "ec3f9f8FF528862aa99Bf4648Fa4844C3d9a50a3",
            "../aligned/keystore0",
            "KEYSTORE_PASSWORD",
            3000000000000,
        )
        .await;
    }
}
