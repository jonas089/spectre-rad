use clap::Parser;
use client::{run, Cli};

pub mod aligned;
mod client;

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();
    run(cli).await;
}

#[cfg(test)]
mod test_risc0 {
    use committee_circuit::{RZ_COMMITTEE_ELF, RZ_COMMITTEE_ID};
    use committee_iso::{
        types::{CommitteeCircuitInput, CommitteeUpdateArgs, Root},
        utils::load_circuit_args_env,
    };
    use risc0_zkvm::{default_prover, ExecutorEnv};

    use crate::aligned::{self, constants::ETH_RPC_URL};
    #[test]
    fn test_committee_circuit_risc0() {
        use std::time::Instant;
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let start_time = Instant::now();
        let committee_update: CommitteeUpdateArgs = load_circuit_args_env();
        let committee_update_inputs: CommitteeCircuitInput = CommitteeCircuitInput {
            pubkeys: committee_update.pubkeys_compressed,
            branch: committee_update.sync_committee_branch,
            state_root: committee_update.finalized_header.state_root.to_vec(),
        };
        let env = ExecutorEnv::builder()
            .write(&committee_update_inputs)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();
        let prove_info = prover.prove(env, RZ_COMMITTEE_ELF).unwrap();
        // A receipt in Risc0 is a wrapper struct around the proof itself and the public journal.
        // Use this for verification with Aligned.
        let receipt = prove_info.receipt;
        let output: Root = receipt.journal.decode().unwrap();
        receipt.verify(RZ_COMMITTEE_ID).unwrap();
        println!("Verified Committee Root: {:?}", &output);
        let duration = start_time.elapsed();
        println!("Elapsed time: {:?}", duration);
    }

    #[tokio::test]
    async fn test_committee_submit_aligned() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let committee_update: CommitteeUpdateArgs = load_circuit_args_env();
        let committee_update_inputs: CommitteeCircuitInput = CommitteeCircuitInput {
            pubkeys: committee_update.pubkeys_compressed,
            branch: committee_update.sync_committee_branch,
            state_root: committee_update.finalized_header.state_root.to_vec(),
        };
        let env = ExecutorEnv::builder()
            .write(&committee_update_inputs)
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
