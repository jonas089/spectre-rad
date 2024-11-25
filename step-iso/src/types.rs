use ethereum_consensus_types::BeaconBlockHeader;
use serde::{Deserialize, Serialize};

pub type PublicKeysUncompressed = Vec<Vec<u8>>;
pub type SignatureCompressed = Vec<u8>;
pub type Branch = Vec<Vec<u8>>;
pub type Root = Vec<u8>;
pub type Leaf = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStepArgs {
    pub signature_compressed: SignatureCompressed,
    // G1Affine
    pub pubkeys_uncompressed: PublicKeysUncompressed,
    pub pariticipation_bits: Vec<bool>,
    pub attested_header: BeaconBlockHeader,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Branch,
    pub execution_payload_root: Root,
    pub execution_payload_branch: Branch,
    pub domain: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStepCircuitOutput {}
