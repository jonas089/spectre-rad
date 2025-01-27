/*
    This main file contains an example execution loop that updates a smart contract on sepolia.
    In production a loop that manages contracts on multiple networks can be used.
    This program serves as a test entry for a production environment.
*/

use alloy_sol_types::SolType;
use committee_iso::utils::{commit_to_keys_with_sign, decode_pubkeys_x};
use ethers::types::Bytes;
use hex::FromHex;
use preprocessor::get_light_client_update_at_slot;
use prover::{eth::SpectreContractClient, generate_rotation_proof_sp1};
use rotation_iso::types::{RotationCircuitInputs, WrappedOutput};
use shiplift::{ContainerListOptions, Docker, RmContainerOptions};
use sp1_sdk::HashableKey;
use std::{process::Command, time::Duration};
use step_iso::types::SyncStepCircuitInput;
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
    let starting_slot = 6823936; //client.read_slot_value().await;
                                 // epoch currently 256
    let target_slot = loop {
        let mut x = 32 * 256;
        while x <= starting_slot {
            x += 32 * 256;
        }
        break x;
    };
    let semaphore = std::sync::Arc::new(Semaphore::new(1));
    loop {
        stop_containers().await;
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
            WrappedOutput::abi_decode(&proof.0.public_values.as_slice(), false).unwrap();
        println!("Commitment: {:?}", &circuit_out.commitment);
        println!("Slot: {:?}", &circuit_out.slot);
        // verify proof on-chain
        /*match client.call_with_args("verifyRotationProof", payload).await {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {}", e);
            }
        }*/
        drop(payload);
        drop(proof);
        println!("Update Success, waiting 10 seconds...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn stop_containers() {
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

/*

untagged: ghcr.io/succinctlabs/sp1-gnark:v4.0.0-rc.3
untagged: ghcr.io/succinctlabs/sp1-gnark@sha256:b9cc16808ad0b7dabb5743cb7f4ac343f40b3485a1c94a38360f40630aab82ab
deleted: sha256:c08c397077eac01ee705d69688ea76c7fe2126005291e76974723b254fd83b82
deleted: sha256:73a03b712244bae7e2d8755339ad986880e453d2d6b0f84a42a3bef9623bd947
deleted: sha256:b3a77f38fae5c013571852095b4fef259650b3dc5b021eb27f01196558e21b92
deleted: sha256:ff80ec55a37609daeaf3d3843aa8105f6c6b18984cf42badda290363444bdef3
*/
