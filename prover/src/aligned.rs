// This file contains a proof of concept interaction between the prover and AlignedLayer
mod constants;
use aligned_sdk::core::types::{Network, PriceEstimate, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{estimate_fee, get_next_nonce, submit_and_wait_verification};
use committee_circuit::RZ_COMMITTEE_ELF;
use constants::{BATCHER_URL, ETH_RPC_URL};
use ethers::types::Address;
use ethers::{signers::LocalWallet, signers::Signer};
use risc0_zkvm::Receipt;

pub(crate) async fn submit_committee_proof(proof: Receipt) {
    const NETWORK: Network = Network::Holesky;
    const KEYSTORE: &str = "../aligned/keystore0";

    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");

    let wallet = LocalWallet::decrypt_keystore(KEYSTORE, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(17000u64);

    let verification_data = VerificationData {
        proving_system: ProvingSystemId::Risc0,
        proof: bincode::serialize(&proof).unwrap(),
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(RZ_COMMITTEE_ELF.to_vec()),
        verification_key: None,
        pub_input: None,
    };

    let max_fee = estimate_fee(ETH_RPC_URL, PriceEstimate::Instant)
        .await
        .expect("failed to fetch gas price from the blockchain");

    match submit_and_wait_verification(
        BATCHER_URL,
        ETH_RPC_URL,
        NETWORK,
        &verification_data,
        max_fee,
        wallet.clone(),
        get_next_nonce(
            ETH_RPC_URL,
            Address::from_slice(&hex::decode("ec3f9f8FF528862aa99Bf4648Fa4844C3d9a50a3").unwrap()),
            NETWORK,
        )
        .await
        .unwrap(),
    )
    .await
    {
        Ok(aligned_verification_data) => {
            println!(
                "Proof submitted and verified successfully on batch {}",
                hex::encode(aligned_verification_data.batch_merkle_root)
            );
        }
        Err(e) => {
            println!("Proof verification failed: {:?}", e);
        }
    }
}
