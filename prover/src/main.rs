/*
    This main file contains an example execution loop that updates a smart contract on sepolia.
    In production a loop that manages contracts on multiple networks can be used.
    This program serves as a test entry for a production environment.
*/

use alloy_sol_types::SolType;
use committee_iso::utils::{commit_to_keys_with_sign, decode_pubkeys_x};
use ethers::types::Bytes;
use hex::FromHex;
use preprocessor::{get_current_sync_step, get_light_client_update_at_slot};
use prover::{eth::SpectreContractClient, generate_rotation_proof_sp1, generate_step_proof_sp1};
use rotation_iso::types::{RotationCircuitInputs, WrappedOutput as WrappedRotationOutput};
use sp1_sdk::HashableKey;
use std::{process::Command, time::Duration};
use step_iso::types::SyncStepCircuitInput;
use step_iso::types::WrappedOutput as WrappedStepOutput;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let abi = r#"[{"inputs":[{"internalType":"address","name":"_verifier","type":"address"},{"internalType":"bytes32","name":"_committeeProgramVKey","type":"bytes32"},{"internalType":"bytes32","name":"_stepProgramVKey","type":"bytes32"},{"internalType":"bytes32","name":"_finalizedHeaderRoot","type":"bytes32"},{"internalType":"bytes32","name":"_activeCommitteeCommitment","type":"bytes32"},{"internalType":"uint32","name":"_activeSlot","type":"uint32"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"activeCommitteeCommitment","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"activeSlot","outputs":[{"internalType":"uint32","name":"","type":"uint32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"committeeProgramVKey","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"finalizedHeaderRoot","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"stepProgramVKey","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"verifier","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes","name":"_publicValues","type":"bytes"},{"internalType":"bytes","name":"_proofBytes","type":"bytes"}],"name":"verifyRotationProof","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"bytes","name":"_publicValues","type":"bytes"},{"internalType":"bytes","name":"_proofBytes","type":"bytes"}],"name":"verifyStepProof","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
    let contract = "0x6B7d0B0681C1f7353b616DaB7c45FDF37e252d4C";
    let rpc_url = dotenv::var("SEPOLIA_RPC_URL").unwrap_or_default();
    let chain_id = 11155111u64;
    let client = SpectreContractClient {
        contract: contract.to_string(),
        abi: serde_json::from_str(&abi).unwrap(),
        rpc_url,
        chain_id,
    };
    let semaphore = std::sync::Arc::new(Semaphore::new(1));
    loop {
        let last_known_slot = client.read_slot_value().await;
        prune_environment().await;
        let sync_step = get_current_sync_step().await;
        let target_slot = u64::from_str_radix(&sync_step.0.attested_header.slot, 10).unwrap();
        if target_slot <= last_known_slot.into() {
            println!("Contract is on top of chain!");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }
        if target_slot % (32 * 256) == 0 {
            let ((s, c), oc) = get_light_client_update_at_slot((target_slot) as u64).await;
            let (keys, signs) = decode_pubkeys_x(oc);
            let commitment = commit_to_keys_with_sign(&keys, &signs);
            println!(
                "Active Committee: {:?}",
                format!("0x{}", hex::encode(commitment))
            );
            println!("Generating Rotation proof at: {}", &target_slot);
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let proof = tokio::task::spawn_blocking(move || {
                let proof = generate_rotation_proof_sp1(
                    &prover::ProverOps::Groth16,
                    RotationCircuitInputs {
                        committee: c,
                        step: SyncStepCircuitInput {
                            args: s,
                            commitment,
                        },
                    },
                );
                drop(permit);
                proof
            })
            .await
            .expect("Prover Task failed!");
            let payload: (Bytes, Bytes) = (
                Bytes::from_hex(&format!(
                    "0x{}",
                    hex::encode(proof.0.public_values.as_slice())
                ))
                .unwrap(),
                Bytes::from_hex(&format!("0x{}", hex::encode(proof.0.bytes()))).unwrap(),
            );
            println!("Verifying Key: {}", hex::encode(&proof.1.bytes32()));
            let circuit_out =
                WrappedRotationOutput::abi_decode(&proof.0.public_values.as_slice(), false)
                    .unwrap();
            println!("Commitment: {:?}", &circuit_out.commitment);
            println!("Slot: {:?}", &circuit_out.slot);
            match client.call_with_args("verifyRotationProof", payload).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        } else {
            println!("Generating Step proof at: {}", &target_slot);
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let proof = tokio::task::spawn_blocking(move || {
                let proof = generate_step_proof_sp1(
                    &prover::ProverOps::Groth16,
                    sync_step.1,
                    sync_step.0,
                    &prover::ProofCompressionBool::Uncompressed,
                );
                drop(permit);
                proof
            })
            .await
            .expect("Prover Task failed!");
            let payload: (Bytes, Bytes) = (
                Bytes::from_hex(&format!(
                    "0x{}",
                    hex::encode(proof.0.public_values.as_slice())
                ))
                .unwrap(),
                Bytes::from_hex(&format!("0x{}", hex::encode(proof.0.bytes()))).unwrap(),
            );
            println!("Verifying Key: {}", hex::encode(&proof.1.bytes32()));
            let circuit_out =
                WrappedStepOutput::abi_decode(&proof.0.public_values.as_slice(), false).unwrap();
            println!("Commitment: {:?}", &circuit_out.commitment);
            println!("Slot: {:?}", &circuit_out.slot);
            match client.call_with_args("verifyStepProof", payload).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

        println!("Update Success, waiting 10 seconds...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn prune_environment() {
    let _ = Command::new("rm")
        .arg("-rf")
        .arg("/Users/chef/.sp1/circuits/groth16/v4.0.0-rc.3")
        .output()
        .expect("Failed to delete artifacts");
    let output = Command::new("docker")
        .arg("system")
        .arg("prune")
        .arg("-a")
        .arg("-f")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("Command executed successfully:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Error executing command:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
