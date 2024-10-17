fn main() {
    todo!("Client");
}

#[cfg(test)]
mod test_risc0 {
    use committee_circuit::{RZ_COMMITTEE_ELF, RZ_COMMITTEE_ID};
    use committee_iso::{
        types::{CommitteeUpdateArgs, Root},
        utils::load_test_args,
    };
    use risc0_zkvm::{default_prover, ExecutorEnv};
    #[test]
    fn test_committee_circuit_risc0() {
        let committee_update: CommitteeUpdateArgs = load_test_args();
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
        let env = ExecutorEnv::builder()
            .write(&committee_update)
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
    }
}
