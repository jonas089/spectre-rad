// This file contains a proof of concept interaction between the prover and AlignedLayer
pub mod constants;
pub mod storage;
use aligned_sdk::core::types::{Network, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{get_next_nonce, submit_and_wait_verification};
use committee_circuit::RZ_COMMITTEE_ID;
use constants::BATCHER_URL;
use ethers::types::{Address, U256};
use ethers::{signers::LocalWallet, signers::Signer};
use risc0_zkvm::Receipt;
use storage::ProofDB;

pub async fn submit_committee_proof(
    proof: Receipt,
    rpc: &str,
    chain_id: u64,
    network: Network,
    address: &str,
    keystore: &str,
    password: &str,
    gas: u64,
) {
    let wallet = LocalWallet::decrypt_keystore(keystore, &password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id);

    let verification_data = VerificationData {
        proving_system: ProvingSystemId::Risc0,
        proof: bincode::serialize(&proof.inner).unwrap(),
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(convert(&RZ_COMMITTEE_ID).to_vec()),
        verification_key: None,
        pub_input: Some(proof.journal.bytes),
    };
    let max_fee = U256::from(gas);

    match submit_and_wait_verification(
        BATCHER_URL,
        &rpc,
        network,
        &verification_data,
        max_fee,
        wallet.clone(),
        get_next_nonce(
            &rpc,
            Address::from_slice(&hex::decode(address).unwrap()),
            network,
        )
        .await
        .unwrap(),
    )
    .await
    {
        Ok(aligned_verification_data) => {
            println!(
                "[Success] Proof was successfully verified and included in batch {}",
                hex::encode(aligned_verification_data.batch_merkle_root)
            );

            let mut db = ProofDB {
                path: "proofs.db".to_string(),
            };
            db.setup();

            let root = aligned_verification_data.batch_merkle_root;
            let height = aligned_verification_data.index_in_batch;

            db.insert(&root, height);
            println!("[Success] Proof was stored in {}", &db.path);

            let verified_proofs = db.get_all();
            println!("[Info] Content of {}", &db.path);

            for proof in verified_proofs {
                println!(
                    "[Proof] \n [Batch Root]: {:?} \n [Batch Height]: {:?}",
                    proof.0, proof.1
                );
            }
        }
        Err(e) => {
            println!("Proof verification failed: {:?}", e);
        }
    }
}

fn convert(data: &[u32; 8]) -> [u8; 32] {
    let mut res = [0; 32];
    for i in 0..8 {
        res[4 * i..4 * (i + 1)].copy_from_slice(&data[i].to_le_bytes());
    }
    res
}
