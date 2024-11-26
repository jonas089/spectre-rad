use aligned_sdk::core::types::Network;
use clap::{Parser, Subcommand};
use committee_circuit::RZ_COMMITTEE_ELF;
use committee_iso::{types::CommitteeUpdateArgs, utils::load_circuit_args};
use prover::aligned;
use risc0_zkvm::{default_prover, ExecutorEnv};

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Prove {
        #[arg(short, long)]
        path: String,
        #[arg(short, long)]
        rpc: String,
        #[arg(short, long)]
        chain_id: u64,
        #[arg(short, long)]
        network: String,
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        keystore: String,
        #[arg(long)]
        password: String,
        #[arg(short, long)]
        gas: u64,
    },
}

pub async fn run(cli: Cli) {
    match cli.command {
        Command::Prove {
            path,
            rpc,
            chain_id,
            network,
            address,
            keystore,
            password,
            gas,
        } => {
            let network_typed = match network.as_str() {
                "Devnet" => Network::Devnet,
                "HoleskyStage" => Network::HoleskyStage,
                _ => Network::Holesky,
            };
            generate_and_submit_committee_proof_aligned(
                &path,
                &rpc,
                chain_id,
                network_typed,
                &address,
                &keystore,
                &password,
                gas,
            )
            .await
        }
    }
}

// this can be replaced/extended with other proofs or a composite proof
// for a full lightclient
async fn generate_and_submit_committee_proof_aligned(
    path: &str,
    rpc: &str,
    chain_id: u64,
    network: Network,
    address: &str,
    keystore: &str,
    password: &str,
    gas: u64,
) {
    let committee_update: CommitteeUpdateArgs = load_circuit_args(path);
    let env = ExecutorEnv::builder()
        .write(&committee_update)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover.prove(env, RZ_COMMITTEE_ELF).unwrap();
    let receipt = prove_info.receipt;
    aligned::submit_committee_proof(
        receipt, rpc, chain_id, network, address, keystore, password, gas,
    )
    .await;
}
