use std::str::FromStr;

use abi::{Abi, ParamType};
use ethers::prelude::*;
use k256::ecdsa::SigningKey;
// query spectre contract and send proof payloads to it

pub struct SpectreContractClient {
    pub contract: String,
    pub abi: Abi,
    pub rpc_url: String,
    pub chain_id: u64,
}

impl SpectreContractClient {
    pub async fn read_slot_value(&self) -> u32 {
        let private_key = dotenv::var("PRIVATE_KEY").unwrap_or_default();
        let wallet: Wallet<SigningKey> = if !private_key.is_empty() {
            private_key
                .parse::<Wallet<SigningKey>>()
                .unwrap()
                .with_chain_id(self.chain_id)
        } else {
            panic!("Missing or Invalid environment variable PRIVATE_KEY!");
        };
        let provider = Provider::<Http>::try_from(&self.rpc_url).unwrap();
        let client = SignerMiddleware::new(provider, wallet).into();
        let contract = Contract::new(
            H160::from_str(&self.contract).unwrap(),
            self.abi.clone(),
            client,
        );

        let value: u32 = contract
            .method::<_, u32>("activeSlot", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        value
    }

    pub async fn call_with_args(
        &self,
        method_name: &str,
        args: (Bytes, Bytes),
    ) -> Result<(), ContractError<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
        let private_key = dotenv::var("PRIVATE_KEY").unwrap_or_default();
        let wallet: Wallet<SigningKey> = if !private_key.is_empty() {
            private_key
                .parse::<Wallet<SigningKey>>()
                .expect("Failed to parse private key!")
                .with_chain_id(self.chain_id)
        } else {
            panic!("Missing or Invalid environment variable PRIVATE_KEY!");
        };

        let provider = Provider::<Http>::try_from(&self.rpc_url).unwrap();
        let client = SignerMiddleware::new(provider, wallet).into();
        let contract = Contract::new(
            H160::from_str(&self.contract).unwrap(),
            self.abi.clone(),
            client,
        );

        // Handle the call with error capture
        match contract.method::<_, ()>(method_name, args)?.send().await {
            Ok(_) => Ok(()),
            Err(error) => {
                // Print or deserialize the Ethereum error
                if let ContractError::Revert(revert_data) = &error {
                    match abi::decode(&[ParamType::String], &revert_data.0) {
                        Ok(decoded) => {
                            println!("Revert Reason: {:?}", decoded[0]);
                        }
                        Err(_) => {
                            println!("Failed to decode revert reason.");
                        }
                    }
                } else {
                    println!("Error: {:?}", error);
                }
                Err(error)
            }
        }
    }
}

#[tokio::test]
async fn test_get_slot() {
    let abi = r#"[{"inputs":[{"internalType":"address","name":"_verifier","type":"address"},{"internalType":"bytes32","name":"_committeeProgramVKey","type":"bytes32"},{"internalType":"bytes32","name":"_stepProgramVKey","type":"bytes32"},{"internalType":"bytes32","name":"_finalizedHeaderRoot","type":"bytes32"},{"internalType":"bytes32","name":"_activeCommitteeCommitment","type":"bytes32"},{"internalType":"uint32","name":"_activeSlot","type":"uint32"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"activeCommitteeCommitment","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"activeSlot","outputs":[{"internalType":"uint32","name":"","type":"uint32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"committeeProgramVKey","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"finalizedHeaderRoot","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"stepProgramVKey","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"verifier","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes","name":"_publicValues","type":"bytes"},{"internalType":"bytes","name":"_proofBytes","type":"bytes"}],"name":"verifyRotationProof","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"bytes","name":"_publicValues","type":"bytes"},{"internalType":"bytes","name":"_proofBytes","type":"bytes"}],"name":"verifyStepProof","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
    let contract = "0xa9a134f006F6e30F3DB1e4a8DdD62B847f905226";
    let rpc_url = dotenv::var("SEPOLIA_RPC_URL").unwrap_or_default();
    let chain_id = 11155111u64;
    let client = SpectreContractClient {
        contract: contract.to_string(),
        abi: serde_json::from_str(&abi).unwrap(),
        rpc_url,
        chain_id,
    };
    let slot: u32 = client.read_slot_value().await;
    println!("Slot: {}", &slot);
}
