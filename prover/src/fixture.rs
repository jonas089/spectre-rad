use std::path::PathBuf;

use alloy_sol_types::SolType;
use committee_iso::types::WrappedOutput as CommitteeWrappedOutput;
use rotation_iso::types::WrappedOutput as RotatoinWrappedOutput;
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};
use step_iso::types::WrappedOutput as StepWrappedOutput;

use crate::ProverOps;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitteeUpdateFixture {
    root: String,
    commitment: String,
    vkey: String,
    public_values: String,
    proof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StepUpdateFixture {
    slot: u32,
    root: String,
    commitment: String,
    vkey: String,
    public_values: String,
    proof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RotationUpdateFixture {
    slot: u32,
    root: String,
    commitment: String,
    next_commitment: String,
    vkey: String,
    public_values: String,
    proof: String,
}

pub fn create_committee_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    ops: &ProverOps,
) {
    let bytes = proof.public_values.as_slice();
    let CommitteeWrappedOutput {
        finalized_header_root,
        commitment,
    } = CommitteeWrappedOutput::abi_decode(bytes, false).unwrap();
    let fixture = CommitteeUpdateFixture {
        root: format!("0x{}", hex::encode(finalized_header_root)),
        commitment: format!("0x{}", hex::encode(commitment)),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };
    let prefix = match ops {
        ProverOps::Default => panic!("No point in generating a fixture for a default proof!"),
        ProverOps::Groth16 => "groth16-fixture.json",
        ProverOps::Plonk => "plonk-fixture.json",
    };
    let fixture_path = PathBuf::from("./");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(prefix),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}

pub fn create_step_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    ops: &ProverOps,
) {
    let bytes = proof.public_values.as_slice();
    let StepWrappedOutput {
        slot,
        finalized_header_root,
        commitment,
    } = StepWrappedOutput::abi_decode(bytes, false).unwrap();
    let fixture = StepUpdateFixture {
        slot,
        root: format!("0x{}", hex::encode(finalized_header_root)),
        commitment: format!("0x{}", hex::encode(commitment)),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };
    let prefix = match ops {
        ProverOps::Default => panic!("No point in generating a fixture for a default proof!"),
        ProverOps::Groth16 => "groth16-fixture.json",
        ProverOps::Plonk => "plonk-fixture.json",
    };
    let fixture_path = PathBuf::from("./");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(prefix),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}

pub fn create_rotation_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    ops: &ProverOps,
) {
    let bytes = proof.public_values.as_slice();
    let RotatoinWrappedOutput {
        slot,
        finalized_header_root,
        commitment,
        next_commitment,
    } = RotatoinWrappedOutput::abi_decode(bytes, false).unwrap();
    let fixture = RotationUpdateFixture {
        slot,
        root: format!("0x{}", hex::encode(finalized_header_root)),
        commitment: format!("0x{}", hex::encode(commitment)),
        next_commitment: format!("0x{}", hex::encode(next_commitment)),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };
    let prefix = match ops {
        ProverOps::Default => panic!("No point in generating a fixture for a default proof!"),
        ProverOps::Groth16 => "groth16-fixture.json",
        ProverOps::Plonk => "plonk-fixture.json",
    };
    let fixture_path = PathBuf::from("./");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(prefix),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
