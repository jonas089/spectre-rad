#[cfg(test)]
mod test_circuits {
    use committee_circuit::RZ_COMMITTEE_ELF;
    use committee_iso::{
        types::CommitteeUpdateArgs, utils::load_circuit_args_env as load_committee_args_env,
    };
    use prover::integrations::aligned::{self, constants::ETH_RPC_URL};
    use risc0_zkvm::{default_prover, ExecutorEnv};

    // Aligned Layer
    #[tokio::test]
    async fn test_committee_submit_aligned() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let committee_update: CommitteeUpdateArgs = load_committee_args_env();
        let env = ExecutorEnv::builder()
            .write_slice(&borsh::to_vec(&committee_update).unwrap())
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
